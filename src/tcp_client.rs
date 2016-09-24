extern crate byteorder;
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

use byteorder::{ByteOrder, BigEndian};
use clap::{App, Arg};
use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

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
    let serverport = matches.value_of("serverport").unwrap().parse::<u16>().unwrap();
    debug!("server port: {}", serverport);
    let udpport = matches.value_of("udpport").unwrap().parse::<u16>().unwrap();
    debug!("udp port: {}", udpport);

    let mut stream = TcpStream::connect((servername, serverport)).unwrap();
    stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();

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

    let (tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();

    thread::spawn(move || client_loop(stream, rx));

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
                        tx.send(station).unwrap();
                    }
                }
            }
            Err(_) => {
                panic!("Unexpected error reading from stdin");
            }
        }
    }
}

fn client_loop(mut stream: TcpStream, rx: Receiver<u16>) {
    loop {
        let station = match rx.try_recv() {
            Ok(station) => station,
            Err(mpsc::TryRecvError::Empty) => 65535,
            Err(mpsc::TryRecvError::Disconnected) => return,
        };

        if station < 65535 {
            let mut setstationbuf = [0u8; 3];
            setstationbuf[0] = 1;
            BigEndian::write_u16(&mut setstationbuf[1..], station);
            debug!("{:?}", setstationbuf);
            stream.write_all(setstationbuf.as_ref()).unwrap();

            println!("Waiting for an announce…");
        }

        let mut reply_type_buf = [0u8; 1];

        // poll server for change of song
        match stream.read_exact(&mut reply_type_buf) {
            Ok(_) => (),
            Err(_) => continue,
        }

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
                print!("> ", );
                io::stdout().flush().unwrap();
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
