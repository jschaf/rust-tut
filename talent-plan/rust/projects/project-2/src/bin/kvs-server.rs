#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use std::env;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;

use clap::{App, AppSettings, Arg};
use slog::Drain;

use kvs;
use kvs::protocol::Command;
use kvs::{KvStore, KvsError};

fn main() -> kvs::Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("addr")
                .long("addr")
                .value_name("ADDR")
                .help("Sets the IP address")
                .default_value("127.0.0.1:4000")
                .validator(kvs::flags::is_valid_ip_addr)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("engine")
                .long("engine")
                .value_name("ENGINE")
                .help("Sets the underlying storage engine")
                .default_value("kvs")
                .validator(kvs::flags::is_valid_engine)
                .takes_value(true),
        )
        .get_matches();

    let addr = SocketAddr::from_str(
        matches
            .value_of("addr")
            .expect("addr not set but should have default"),
    )
    .expect("addr should be validated as valid IP");
    let engine = matches
        .value_of("engine")
        .expect("engine not set but should have default");

    let log = init_logger();
    info!(log, "kvs-server started on {}", addr);
    info!(log, "version: {}", env!("CARGO_PKG_VERSION"));
    info!(log, "engine: {}", engine);

    let listener = TcpListener::bind(addr)?;

    let size_buffer = [0u8; 4];
    let mut bytes_buffer = Vec::with_capacity(256);

    let mut store = KvStore::open(env::current_dir()?)?;

    for stream in listener.incoming() {
        let mut stream = stream?;
        let cmd = read_cmd(&mut stream, size_buffer, &mut bytes_buffer)?;
        info!(log, "Read command {:?}", cmd);
        match cmd {
            Command::Get(key) => match store.get(key) {
                Ok(Some(value)) => stream.write(&Command::GetResponse(value).serialize()),
                Ok(None) => stream.write(&Command::NotFound.serialize()),
                Err(msg) => stream.write(&Command::Error(String::from("error")).serialize()),
            },
            Command::Set(key, value) => match store.set(key, value) {
                Ok(()) => stream.write(&Command::SetResponse.serialize()),
                Err(msg) => stream.write(&Command::Error(String::from("error")).serialize()),
            },
            Command::Remove(key) => match store.remove(key) {
                Ok(()) => stream.write(&Command::RemoveResponse.serialize()),
                Err(msg) => stream.write(&Command::Error(String::from("error")).serialize()),
            },
            _ => Ok(1),
        };
    }
    Ok(())
}

fn read_cmd(
    stream: &mut TcpStream,
    mut size_buffer: [u8; 4],
    bytes_buffer: &mut Vec<u8>,
) -> Result<Command, KvsError> {
    stream.peek(&mut size_buffer)?;
    let size = u32::from_le_bytes(size_buffer) as usize;
    bytes_buffer.clear();
    bytes_buffer.resize(size, 0);
    stream.read_exact(bytes_buffer)?;
    Command::deserialize(&bytes_buffer).map_err(KvsError::Protocol)
}

fn init_logger() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, o!())
}
