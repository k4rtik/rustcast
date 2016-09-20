extern crate clap;

use clap::{App, Arg};

fn main() {
    App::new("snowcast_control")
        .version("0.1.0")
        .arg(Arg::with_name("servername").required(true).index(1))
        .arg(Arg::with_name("serverport").required(true).index(2))
        .arg(Arg::with_name("udpport").required(true).index(3))
        .get_matches();
}
