extern crate clap;

use clap::{App, Arg};

fn main() {
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
