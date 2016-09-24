use std::io;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::net::Ipv4Addr;

use byteorder::{ByteOrder, BigEndian};
use commands::*;

use mio::*;
use mio::tcp::*;

/// A stateful wrapper around a non-blocking stream. This connection is not
/// the SERVER connection. This connection represents the client connections
/// _accepted_ by the SERVER connection.
pub struct Connection {
    // handle to the accepted socket
    sock: TcpStream,

    // token used to register with the poller
    pub token: Token,

    // set of events we are interested in
    interest: Ready,

    // messages waiting to be sent out
    send_queue: Vec<Rc<Vec<u8>>>,

    // track whether a connection needs to be (re)registered
    is_idle: bool,

    // track whether a connection is reset
    is_reset: bool,

    is_to_be_removed: bool,

    handshake_done: bool,

    current_channel: u16,

    udp_port: u16,

    addr: Ipv4Addr,
}

impl Connection {
    pub fn new(sock: TcpStream, token: Token, addr: Ipv4Addr) -> Connection {
        Connection {
            sock: sock,
            token: token,
            interest: Ready::hup(),
            send_queue: Vec::new(),
            is_idle: true,
            is_reset: false,
            is_to_be_removed: false,
            handshake_done: false,
            current_channel: 65535,
            udp_port: 0,
            addr: addr,
        }
    }

    /// Handle read event from poller.
    ///
    /// The Handler must continue calling until None is returned.
    ///
    /// The receive buffer is sent back to `Server` so the message can be broadcast to all
    /// listening connections.
    pub fn readable(&mut self) -> io::Result<Option<ServerCommand>> {
        self.read_command()
    }

    fn read_command(&mut self) -> io::Result<Option<ServerCommand>> {
        let mut buf = [0u8; 3];

        let bytes = match self.sock.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }
        };

        if bytes < 3 {
            warn!("Found message length of {} bytes", bytes);
            return Err(Error::new(ErrorKind::InvalidData, "Invalid message length"));
        }

        let command_type = buf[0];
        let command_value = BigEndian::read_u16(&buf[1..]);
        let command = match command_type {
            0 => {
                ServerCommand::Hello {
                    command_type: command_type,
                    udp_port: command_value,
                }
            }
            1 => {
                ServerCommand::SetStation {
                    command_type: command_type,
                    station_number: command_value,
                }
            }
            _ => {
                ServerCommand::Invalid {
                    command_type: 99,
                    unused: 99,
                }
            }
        };

        Ok(Some(command))
    }

    /// Handle a writable event from the poller.
    ///
    /// Send one message from the send queue to the client. If the queue is empty, remove interest
    /// in write events.
    pub fn writable(&mut self) -> io::Result<()> {

        try!(self.send_queue
            .pop()
            .ok_or(Error::new(ErrorKind::Other, "Could not pop send queue"))
            .and_then(|buf| {

                match self.sock.write(&*buf) {
                    Ok(n) => {
                        debug!("CONN : we wrote {} bytes", n);
                        Ok(())
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            debug!("client flushing buf; WouldBlock");

                            // put message back into the queue so we can try again
                            self.send_queue.push(buf);
                            Ok(())
                        } else {
                            error!("Failed to send buffer for {:?}, error: {}", self.token, e);
                            Err(e)
                        }
                    }
                }
            }));

        if self.send_queue.is_empty() {
            self.interest.remove(Ready::writable());
        }

        if self.is_to_be_removed() {
            debug!("Marking for reset: {:?}", self.token);
            self.mark_reset();
        }

        Ok(())
    }

    /// Queue an outgoing message to the client.
    ///
    /// This will cause the connection to register interests in write events with the poller.
    /// The connection can still safely have an interest in read events. The read and write buffers
    /// operate independently of each other.
    pub fn send_message(&mut self, message: Rc<Vec<u8>>) -> io::Result<()> {
        trace!("connection send_message; token={:?}", self.token);

        self.send_queue.push(message);

        if !self.interest.is_writable() {
            self.interest.insert(Ready::writable());
        }

        Ok(())
    }

    /// Register interest in read events with poll.
    ///
    /// This will let our connection accept reads starting next poller tick.
    pub fn register(&mut self, poll: &mut Poll) -> io::Result<()> {
        trace!("connection register; token={:?}", self.token);

        self.interest.insert(Ready::readable());

        poll.register(&self.sock,
                      self.token,
                      self.interest,
                      PollOpt::edge() | PollOpt::oneshot())
            .and_then(|()| {
                self.is_idle = false;
                Ok(())
            })
            .or_else(|e| {
                error!("Failed to reregister {:?}, {:?}", self.token, e);
                Err(e)
            })
    }

    /// Re-register interest in read events with poll.
    pub fn reregister(&mut self, poll: &mut Poll) -> io::Result<()> {
        trace!("connection reregister; token={:?}", self.token);

        poll.reregister(&self.sock,
                        self.token,
                        self.interest,
                        PollOpt::edge() | PollOpt::oneshot())
            .and_then(|()| {
                self.is_idle = false;
                Ok(())
            })
            .or_else(|e| {
                error!("Failed to reregister {:?}, {:?}", self.token, e);
                Err(e)
            })
    }

    pub fn mark_reset(&mut self) {
        trace!("connection mark_reset; token={:?}", self.token);

        self.is_reset = true;
    }

    #[inline]
    pub fn is_reset(&self) -> bool {
        self.is_reset
    }

    pub fn mark_idle(&mut self) {
        trace!("connection mark_idle; token={:?}", self.token);

        self.is_idle = true;
    }

    #[inline]
    pub fn is_idle(&self) -> bool {
        self.is_idle
    }

    pub fn mark_to_be_removed(&mut self) {
        trace!("connection mark_to_be_removed; token={:?}", self.token);

        self.is_to_be_removed = true;
    }

    #[inline]
    pub fn is_to_be_removed(&self) -> bool {
        self.is_to_be_removed
    }

    pub fn mark_handshake_done(&mut self) {
        trace!("connection handshake_done; token={:?}", self.token);

        self.handshake_done = true;
    }

    #[inline]
    pub fn is_handshake_done(&self) -> bool {
        self.handshake_done
    }

    pub fn set_current_channel(&mut self, channel: u16) {
        self.current_channel = channel;
    }

    #[inline]
    pub fn get_current_channel(&self) -> u16 {
        self.current_channel
    }

    pub fn set_udp_port(&mut self, port: u16) {
        self.udp_port = port;
    }

    #[inline]
    pub fn get_udp_port(&self) -> u16 {
        self.udp_port
    }

    #[inline]
    pub fn get_addr(&self) -> Ipv4Addr {
        self.addr
    }
}
