extern crate byteorder;
extern crate clap;
extern crate mio;

#[macro_use]
extern crate log;
extern crate env_logger;

mod commands;

use byteorder::{ByteOrder, BigEndian};
use clap::{App, Arg};
use commands::*;
use mio::*;
use mio::tcp::TcpStream;
use std::io;
use std::io::prelude::*;
use std::os::unix::io::FromRawFd;

const CONTROL: Token = Token(0);
const CLIENT: Token = Token(1);

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
    debug!("server: {}", servername);
    let serverport = matches.value_of("serverport").unwrap(); // .parse::<u16>().unwrap();
    debug!("server port: {}", serverport);
    let udpport = matches.value_of("udpport").unwrap().parse::<u16>().unwrap();
    debug!("udp port: {}", udpport);

    let addr_string = servername.to_string() + ":" + serverport;
    let addr = addr_string.parse().unwrap();
    let sock = TcpStream::connect(&addr).unwrap();

    let mut poll = Poll::new().unwrap();
    poll.register(&sock, CLIENT, Ready::readable(), PollOpt::edge()).unwrap();

    let control = unsafe { std::os::unix::io::from_raw_fd(0) };
    poll.register(&control, CONTROL, Ready::readable(), PollOpt::edge()).unwrap();

    let mut events = Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                CONTROL => {}
                CLIENT => {}
                _ => unreachable!(),
            }
        }
    }

    let mut hellobuf = [0u8; 3];
    BigEndian::write_u16(&mut hellobuf[1..], udpport);
    debug!("{:?}", hellobuf);
    stream.write_all(hellobuf.as_ref()).unwrap();

    let mut welcomebuf = [0u8; 3];
    stream.read_exact(&mut welcomebuf).unwrap();
    let reply_type = welcomebuf[0];
    let num_stations = BigEndian::read_u16(&welcomebuf[1..]);
    info!("reply_type: {}, num_stations: {}", reply_type, num_stations);

    println!("Type in a number to set the station we're listening to to that number.");
    println!("Enter q or press CTRL+C to quit.");
    println!("> The server has {} stations.", num_stations);

    poll.register(&stream, CLIENT, Ready::all(), PollOpt::edge()).unwrap();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                info!("EndOfFile sent (Ctrl-D)");
                break;
            }
            Ok(_) => {
                match input.trim() {
                    "q" => break,
                    x => {
                        let stationres = x.parse::<u16>();
                        let station = match stationres {
                            Ok(num) => num,
                            Err(_) => {
                                println!("Invalid input: number or 'q' expected");
                                continue;
                            }
                        };

                        let mut setstationbuf = [0u8; 3];
                        setstationbuf[0] = 1;
                        BigEndian::write_u16(&mut setstationbuf[1..], station);
                        debug!("{:?}", setstationbuf);
                        stream.write_all(setstationbuf.as_ref()).unwrap();

                        println!("Waiting for an announce…");

                        let mut reply_type_buf = [0u8; 1];
                        stream.read_exact(&mut reply_type_buf).unwrap();
                        info!("{}", reply_type_buf[0]);
                        match reply_type_buf[0] {
                            0 => {
                                error!("Server resent Welcome");
                                break;
                            }
                            1 => {
                                debug!("Announce");
                                let mut song_name_size = [0u8; 1];
                                stream.read_exact(&mut song_name_size).unwrap();
                                info!("{}", song_name_size[0]);
                                let song_name_size = song_name_size[0] as usize;
                                let mut song_name = vec![0u8; song_name_size];
                                stream.read_exact(&mut song_name).unwrap();

                                println!("New song announced: {}",
                                         String::from_utf8(song_name).unwrap());
                            }
                            2 => {
                                // client sent an InvalidCommand
                                let mut reply_string_size = [0u8; 1];
                                stream.read_exact(&mut reply_string_size).unwrap();
                                info!("{}", reply_string_size[0]);
                                let mut reply_string = String::new();
                                assert_eq!(reply_string_size[0] as usize,
                                           stream.read_to_string(&mut reply_string).unwrap());
                                info!("{}", reply_string);

                                println!("INVALID_COMMAND_REPLY: {}", reply_string);
                                println!("Server has closed the connection.");
                                break;
                            }
                            _ => {
                                error!("Server sent an unknown response");
                                break;
                            }

                        };
                    }
                }
            }
            Err(_) => {
                panic!("Unexpected error reading from stdin");
            }
        }
    }
}
