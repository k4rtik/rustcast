extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{App, Arg};
use std::net::UdpSocket;
use std::io::{self, Write};

fn main() {
    env_logger::init().ok().expect("Failed to initialize logger");

    let matches = App::new("snowcast_listener")
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
        // TODO find out optimal buffer window to read
        let mut buf = [0u8; 1400];
        let (amt, _) = socket.recv_from(&mut buf).unwrap();
        // TODO do I need a timeout here?

        io::stdout().write(&buf[0..amt]).unwrap();
    }
}
