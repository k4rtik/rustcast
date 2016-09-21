extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{App, Arg};

fn main() {
    env_logger::init().ok().expect("Failed to initialize logger");

    let matches = App::new("snowcast_listener")
        .version("0.1.0")
        .arg(Arg::with_name("udpport")
            .required(true)
            .index(1)
            .help("e.g. any port between 16384-16387"))
        .get_matches();
}
