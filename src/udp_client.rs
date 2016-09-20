extern crate clap;

use clap::{App, Arg};

fn main() {
    App::new("snowcast_listener")
        .version("0.1.0")
        .arg(Arg::with_name("udpport").required(true).index(1))
        .get_matches();
}
