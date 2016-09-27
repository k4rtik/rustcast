extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{App, Arg};
use std::net::UdpSocket;
use std::io::{self, Write};

fn main() {
    env_logger::init().ok().expect("Failed to initialize logger");

    let matches = App::new("rustcast_listener")
        .version("0.1.0")
        .arg(Arg::with_name("udpport")
            .required(true)
            .index(1)
            .help("e.g. any port between 16384-16387"))
        .get_matches();

    let port = matches.value_of("udpport").unwrap().parse::<u16>().unwrap();
    info!("udpport: {}", port);

    let socket = UdpSocket::bind(("0.0.0.0", port)).unwrap();

    loop {
        let mut buf = [0u8; 2048]; // unsure if this should match the server buffer size
        let (amt, _) = socket.recv_from(&mut buf).unwrap();
        io::stdout().write(&buf[0..amt]).unwrap();
    }
}
