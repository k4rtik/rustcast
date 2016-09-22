extern crate clap;
extern crate mio;
extern crate slab;

#[macro_use]
extern crate log;
extern crate env_logger;

mod commands;
mod connection;

use clap::{App, Arg};
use commands::*;
use connection::Connection;
use mio::*;
use mio::tcp::*;
use std::net::SocketAddr;
use std::io::{self, ErrorKind};
use std::rc::Rc;

type Slab<T> = slab::Slab<T, Token>;

// stores server-side state
pub struct Server {
    socket: TcpListener,
    token: Token,
    // token to connection map
    conns: Slab<Connection>,
    events: Events,
}

impl Server {
    pub fn new(socket: TcpListener) -> Server {
        Server {
            socket: socket,
            token: Token(10_000_000),

            // SERVER is Token(1), so start after that
            // we can deal with a max of 126 connections
            // TODO change to vec! to allow unlimited connections?
            conns: Slab::with_capacity(128),

            // list of events from the poller that the server needs to process
            events: Events::with_capacity(1024),
        }
    }

    pub fn register(&mut self, poll: &mut Poll) -> io::Result<()> {
        poll.register(&self.socket, self.token, Ready::readable(), PollOpt::edge())
            .or_else(|e| {
                error!("Failed to register server {:?}, {:?}", self.token, e);
                Err(e)
            })
    }

    fn tick(&mut self, poll: &mut Poll) {
        trace!("Handling end of tick");

        let mut reset_tokens = Vec::new();

        for c in self.conns.iter_mut() {
            if c.is_reset() {
                reset_tokens.push(c.token);
            } else if c.is_idle() {
                c.reregister(poll)
                    .unwrap_or_else(|e| {
                        warn!("Reregister failed {:?}", e);
                        c.mark_reset();
                        reset_tokens.push(c.token);
                    });
            }
        }

        for token in reset_tokens {
            match self.conns.remove(token) {
                Some(_c) => {
                    debug!("reset connection; token={:?}", token);
                }
                None => {
                    warn!("Unable to remove connection for {:?}", token);
                }
            }
        }
    }

    pub fn run(&mut self, poll: &mut Poll) -> io::Result<()> {

        try!(self.register(poll));

        info!("Server run loop starting...");
        loop {
            let cnt = try!(poll.poll(&mut self.events, None));

            let mut i = 0;

            trace!("processing events... cnt={}; len={}",
                   cnt,
                   self.events.len());
            while i < cnt {
                let event = self.events.get(i).expect("Failed to get event");

                trace!("event={:?}; idx={:?}", event, i);
                self.ready(poll, event.token(), event.kind());

                i += 1;
            }

            self.tick(poll);
        }
    }

    fn get_connection<'a>(&'a mut self, token: Token) -> &'a mut Connection {
        &mut self.conns[token]
    }

    fn accept(&mut self, poll: &mut Poll) {
        debug!("server accepting new socket");

        loop {
            let sock = match self.socket.accept() {
                Ok((sock, _)) => sock,
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        // equivalent to EAGAIN
                        debug!("accept encountered WouldBlock");
                    } else {
                        error!("Failed to accept new socket, {:?}", e);
                    }
                    return;
                }
            };

            let token = match self.conns.vacant_entry() {
                Some(entry) => {
                    debug!("registering {:?} with poller", entry.index());
                    let c = Connection::new(sock, entry.index());
                    entry.insert(c).index()
                }
                None => {
                    error!("Failed to insert connection into slab");
                    return;
                }
            };

            match self.get_connection(token).register(poll) {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to register {:?} connection with poller, {:?}",
                           token,
                           e);
                    self.conns.remove(token);
                }
            }
        }
    }

    fn readable(&mut self, token: Token) -> io::Result<()> {
        debug!("server conn readable; token={:?}", token);

        while let Some(message) = try!(self.get_connection(token).readable()) {

            let rc_message = Rc::new(message);
            // Queue up a write for all connected clients.
            for c in self.conns.iter_mut() {
                c.send_message(rc_message.clone())
                    .unwrap_or_else(|e| {
                        error!("Failed to queue message for {:?}: {:?}", c.token, e);
                        c.mark_reset();
                    });
            }
        }

        Ok(())
    }

    // here poll is like event loop
    fn ready(&mut self, poll: &mut Poll, token: Token, event: Ready) {
        debug!("token: {:?}, event: {:?}", token, event);

        if event.is_error() {
            warn!("Error event for {:?}", token);
            self.get_connection(token).mark_reset();
            return;
        }

        if event.is_hup() {
            trace!("Hup event for {:?}", token);
            self.get_connection(token).mark_reset();
            return;
        }

        if event.is_writable() {
            trace!("Write event for {:?}", token);
            assert!(self.token != token, "Received writable event for Server");

            let conn = self.get_connection(token);

            if conn.is_reset() {
                info!("{:?} has already been reset", token);
                return;
            }

            conn.writable()
                .unwrap_or_else(|e| {
                    warn!("Write event failed for {:?}, {:?}", token, e);
                    conn.mark_reset();
                });
        }

        // read event
        if event.is_readable() {
            trace!("Read event for {:?}", token);
            if self.token == token {
                // new client connection coming in
                // this accept wrapper will setup necessary serverside state as well
                self.accept(poll);
            } else {
                if self.get_connection(token).is_reset() {
                    info!("{:?} has already been reset", token);
                    return;
                }

                // forward read event to existing connection
                self.readable(token)
                    .unwrap_or_else(|e| {
                        warn!("Read event failed for {:?}: {:?}", token, e);
                        self.get_connection(token).mark_reset();
                    });
            }
        }

        if self.token != token {
            self.get_connection(token).mark_idle();
        }
    }
}

fn main() {
    env_logger::init().ok().expect("Failed to initialize logger");

    let matches = App::new("snowcast_server")
        .version("0.1.0")
        .arg(Arg::with_name("tcpport").required(true).index(1).help("e.g.: 8001"))
        .arg(Arg::with_name("file1")
            .required(true)
            .index(2)
            .help("e.g.: ../mp3/U2-StuckInAMoment.mp3 OR ../mp3/* (to glob)")
            .multiple(true))
        .get_matches();

    let serverport = matches.value_of("tcpport").unwrap();
    debug!("port: {}", serverport);
    let addr = ("127.0.0.1:".to_string() + serverport)
        .parse::<SocketAddr>()
        .ok()
        .expect("Failed to parse port");
    let sock = TcpListener::bind(&addr).ok().expect("Failed to bind address");

    let mut poll = Poll::new().expect("Failed to create Poll");

    let mut server = Server::new(sock);
    server.run(&mut poll).expect("Failed to run server");
}
