extern crate byteorder;
extern crate clap;
extern crate mio;
extern crate slab;

#[macro_use]
extern crate log;
extern crate env_logger;

mod commands;
mod server;
mod connection;

use clap::{App, Arg};
use mio::*;
use mio::tcp::*;
use server::*;
use std::net::SocketAddr;

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

    let mut stations: Vec<String> = vec![];
    if let Some(files) = matches.values_of("file1") {
        for file in files {
            stations.push(file.parse::<String>().unwrap());
        }
    }
    debug!("{:?}", stations);

    let serverport = matches.value_of("tcpport").unwrap();
    debug!("server port: {}", serverport);

    let addr = ("127.0.0.1:".to_string() + serverport)
        .parse::<SocketAddr>()
        .ok()
        .expect("Failed to parse host:port string");
    let sock = TcpListener::bind(&addr).ok().expect("Failed to bind address");

    // Create a polling object that will be used by the server to receive events
    let mut poll = Poll::new().expect("Failed to create Poll");

    // Create our Server object and start polling for events. I am hiding away
    // the details of how registering works inside of the `Server` object. One reason I
    // really like this is to get around having to have `const SERVER = Token(0)` at the top of my
    // file. It also keeps our polling options inside `Server`.
    let mut server = Server::new(sock, stations);
    server.run(&mut poll).expect("Failed to run server");
}
