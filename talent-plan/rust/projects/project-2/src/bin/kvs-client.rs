use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;

use clap::{App, AppSettings, Arg, SubCommand};

use kvs;
use kvs::protocol::Command;

fn main() -> kvs::Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("addr")
                .long("addr")
                .value_name("ADDR")
                .help("Sets the IP address")
                .default_value("127.0.0.1:4000")
                .validator(kvs::flags::is_valid_ip_addr)
                .global(true)
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("Set the value of a string key to a string")
                .arg(Arg::with_name("KEY").help("A string key").required(true))
                .arg(
                    Arg::with_name("VALUE")
                        .help("The string value of the key")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get the string value of a given string key")
                .arg(Arg::with_name("KEY").help("A string key").required(true)),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Remove a given key")
                .arg(Arg::with_name("KEY").help("A string key").required(true)),
        )
        .get_matches();

    let addr = SocketAddr::from_str(
        matches
            .value_of("addr")
            .expect("addr not set but should have default"),
    )
    .expect("Already validated socket addr");

    let mut stream = TcpStream::connect(addr)?;

    match matches.subcommand() {
        ("set", Some(matches)) => {
            let key = matches.value_of("KEY").expect("KEY argument missing");
            let value = matches.value_of("VALUE").expect("VALUE argument missing");
            stream.write(&Command::Set(String::from(key), String::from(value)).serialize())?;
            let cmd = read_next_cmd(&mut stream)?;
            match cmd {
                Command::SetResponse => {}
                Command::Error(msg) => eprintln!("Error: {:?}", msg),
                _ => unreachable!("Unexpected cmd: {:?}", cmd),
            }
        }
        ("get", Some(matches)) => {
            let key = matches.value_of("KEY").expect("KEY argument missing");
            stream.write(&Command::Get(String::from(key)).serialize())?;
            let cmd = read_next_cmd(&mut stream)?;
            match cmd {
                Command::GetResponse(value) => println!("{}", value),
                Command::NotFound => {
                    println!("Key not found");
                }
                Command::Error(msg) => eprintln!("Error: {:?}", msg),
                _ => unreachable!("Unexpected cmd: {:?}", cmd),
            }
        }
        ("rm", Some(matches)) => {
            let key = matches.value_of("KEY").expect("KEY argument missing");
            stream.write(&Command::Remove(String::from(key)).serialize())?;
            let cmd = read_next_cmd(&mut stream)?;
            match cmd {
                Command::RemoveResponse => {}
                Command::Error(msg) => eprintln!("Error: {:?}", msg),
                _ => unreachable!("Unexpected cmd: {:?}", cmd),
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn read_next_cmd(stream: &mut TcpStream) -> Result<Command, kvs::KvsError> {
    let mut size_buffer = [0u8; 4];
    let mut bytes_buffer = Vec::with_capacity(256);

    stream.peek(&mut size_buffer)?;
    let size = u32::from_le_bytes(size_buffer) as usize;
    bytes_buffer.clear();
    bytes_buffer.resize(size, 0);
    stream.read_exact(&mut bytes_buffer)?;
    let cmd = Command::deserialize(&bytes_buffer).map_err(kvs::KvsError::Protocol);
    eprintln!("Read command {:?}", cmd);
    return cmd;
}
