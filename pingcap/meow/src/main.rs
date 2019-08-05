extern crate clap;

use std::env;

use clap::{App, Arg, SubCommand};

fn main() {
    let args: Vec<String> = env::args().collect();
    let matches = App::new("meow")
        .version("1.0")
        .author("Joe Schafer")
        .about("Does meow")
        .arg(
            Arg::with_name("sound")
                .short("c")
                .long("sound")
                .value_name("SOUND")
                .default_value("meow")
                .help("Sets a custom sound instead of meow")
                .takes_value(true),
        )
        .get_matches();
    println!(
        "Making sound: {}",
        matches.value_of("sound").unwrap_or("meow")
    );
}
