extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{App, Arg};

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

    if let Some(mp3s) = matches.values_of("file1") {
        for mp3 in mp3s {
            println!("Received: {}", mp3);
        }
    }
}
