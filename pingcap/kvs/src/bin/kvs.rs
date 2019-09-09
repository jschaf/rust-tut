use std::{env, process};

use clap::{App, AppSettings, Arg, SubCommand};

use kvs::{KvStore, KvsError};

fn main() -> Result<(), KvsError> {
    eprintln!("!! main");
    let matches = App::new("kvs")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("version").short("V").long("version"))
        .subcommand(
            SubCommand::with_name("get").about("Get a key").arg(
                Arg::with_name("KEY")
                    .help("gets a key")
                    .required(true)
                    .index(1),
            ),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("sets a key to a value")
                .arg(
                    Arg::with_name("KEY")
                        .help("The key to get")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("VALUE")
                        .help("The value to set for a key")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("rm").about("removes a key").arg(
                Arg::with_name("KEY")
                    .help("The key to get")
                    .required(true)
                    .index(1),
            ),
        )
        .get_matches();

    match matches.subcommand() {
        ("", None) => {
            if matches.is_present("version") {
                println!(env!("CARGO_PKG_VERSION"));
            }
        }
        ("version", Some(_)) => {
            println!(env!("CARGO_PKG_VERSION"));
        }
        ("get", Some(matches)) => {
            let store = KvStore::open(env::current_dir()?)?;
            let key = matches.value_of("KEY").expect("KEY must be set");
            match store.get(String::from(key))? {
                None => println!("Key not found"),
                Some(val) => {
                    eprintln!("GOT KEY");
                    eprintln!("{}", val);
                    println!("{}", val);
                }
            };
        }
        ("set", Some(matches)) => {
            let mut store = KvStore::open(env::current_dir()?)?;
            let key = matches.value_of("KEY").expect("KEY must be set");
            let value = matches.value_of("VALUE").expect("VALUE must be set");
            store.set(String::from(key), String::from(value))?;
        }
        ("rm", Some(matches)) => {
            eprintln!("# Removing key");
            let key = matches.value_of("KEY").expect("KEY must be set");
            let mut store = KvStore::open(env::current_dir()?)?;
            match store.remove(String::from(key)) {
                Ok(()) => {}
                Err(KvsError::KeyNotFound) => {
                    println!("Key not found");
                    //                    process::exit(1);
                }
                Err(e) => return Err(e),
            }
        }
        _ => {}
    }

    Ok(())
}
