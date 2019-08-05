use std::error::Error;
use std::fs;
use std::{env, process};

use minigrep::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::parse_from_env(args.as_slice()).unwrap_or_else(|err| {
        println!("failed to read args: {}", err);
        process::exit(1)
    });

    let query = &args[1];
    let filename = &args[2];

    println!("Searching for {}", query);
    println!("In file {}", filename);
    if let Err(e) = minigrep::run(config) {
        println!("app error: {}", e);
        process::exit(1);
    }
}
