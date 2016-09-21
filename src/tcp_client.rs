extern crate clap;

use clap::{App, Arg};

fn main() {
    App::new("snowcast_control")
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
}
