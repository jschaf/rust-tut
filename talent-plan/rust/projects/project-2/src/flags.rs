//! Utilities for working with flags.
use std::net::SocketAddr;
use std::str::FromStr;

/// Checks if addr is a valid socket address.
pub fn is_valid_ip_addr(addr: String) -> Result<(), String> {
    match SocketAddr::from_str(addr.as_str()) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Unable to parse: '{}'", addr)),
    }
}

/// Checks if engine is a supported engine.
pub fn is_valid_engine(engine: String) -> Result<(), String> {
    if vec!["kvs", "sled"].contains(&engine.as_str()) {
        return Ok(());
    }
    Err(String::from(
        "Only 'kvs' or 'sled' are supported engine types",
    ))
}
