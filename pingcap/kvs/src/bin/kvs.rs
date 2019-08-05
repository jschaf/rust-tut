use clap::{App, AppSettings, Arg, SubCommand};
use kvs::KvStore;

fn main() {
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

    let mut store = KvStore::new();

    if matches.is_present("version") {
        println!(env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    } else if let Some(matches) = matches.subcommand_matches("get") {
        let key = matches.value_of("KEY").expect("KEY must be set");
        store.get(String::from(key));
    } else if let Some(matches) = matches.subcommand_matches("set") {
        let key = matches.value_of("KEY").expect("KEY must be set");
        let value = matches.value_of("VALUE").expect("VALUE must be set");
        store.set(String::from(key), String::from(value));
    } else if let Some(matches) = matches.subcommand_matches("rm") {
        let key = matches.value_of("KEY").expect("KEY must be set");
        store.remove(String::from(key));
    }
}
