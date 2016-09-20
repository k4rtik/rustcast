extern crate clap;

use clap::{App, Arg};

fn main() {
    let matches = App::new("snowcast_server")
        .version("0.1.0")
        .arg(Arg::with_name("port").required(true).index(1))
        .arg(Arg::with_name("file(s)").required(true).index(2).multiple(true))
        .get_matches();

    if let Some(mp3s) = matches.values_of("file(s)") {
        for mp3 in mp3s {
            println!("Received: {}", mp3);
        }
    }
}
