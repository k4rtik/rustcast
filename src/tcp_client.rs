extern crate byteorder;
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

mod commands;

use byteorder::{ByteOrder, BigEndian};
use clap::{App, Arg};
use commands::*;
use std::io::prelude::*;
use std::net::TcpStream;


fn main() {
    env_logger::init().ok().expect("Failed to initialize logger");

    let matches = App::new("snowcast_control")
        .version("0.1.0")
        .arg(Arg::with_name("servername")
            .required(true)
            .index(1)
            .help("e.g. localhost OR 10.116.70.158"))
        .arg(Arg::with_name("serverport").required(true).index(2).help("e.g. 8001"))
        .arg(Arg::with_name("udpport")
            .required(true)
            .index(3)
            .help("e.g. any port between 16384-16387"))
        .get_matches();

    let servername = matches.value_of("servername").unwrap();
    info!("server: {}", servername);
    let serverport = matches.value_of("serverport").unwrap().parse::<u16>().unwrap();
    info!("server port: {}", serverport);
    let udpport = matches.value_of("udpport").unwrap().parse::<u16>().unwrap();
    info!("udp port: {}", udpport);

    let mut stream = TcpStream::connect((servername, serverport)).unwrap();

    let mut hellobuf = [0u8; 3];
    let hello = Hello {
        command_type: 0,
        udp_port: udpport,
    };
    hellobuf[0] = hello.command_type;
    BigEndian::write_u16(&mut hellobuf[1..], hello.udp_port);
    info!("{:?}", hellobuf);
    stream.write_all(hellobuf.as_ref()).unwrap();

    let mut welcomebuf = [0u8; 3];
    stream.read_exact(&mut welcomebuf).unwrap();
    let welcome = Welcome {
        reply_type: welcomebuf[0],
        num_stations: BigEndian::read_u16(&welcomebuf[1..]),
    };
    info!("{} {}", welcome.reply_type, welcome.num_stations);

    loop {

    }
}
