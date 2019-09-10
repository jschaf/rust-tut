use redis_ponger::redis::RedisType;
use redis_ponger::serde_resp;
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:6379")?;
    let cmd = RedisType::Array(vec![RedisType::BulkString("ping".to_string())]);
    let result1 = serde_resp::to_string(&cmd).unwrap();
    eprintln!("result: {}", result1);
    stream.write_all(result1.as_bytes())?;
    let mut result = String::new();
    stream.take(4).read_to_string(&mut result)?;
    println!("{}", "Recv Bytes");
    println!("{}", "=====");
    println!("{:?}", result);
    println!("{}", "=====");
    eprintln!("Finished");

    Ok(())
}
// For Simple Strings the first byte of the reply is "+"
// For Errors the first byte of the reply is "-"
// For Integers the first byte of the reply is ":"
// For Bulk Strings the first byte of the reply is "$"
// For Arrays the first byte of the reply is "*"
