//! The wire protocol.

use std::str;

/// A command to send to the key-value store.
#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    /// Get the value for a String key.
    Get(String),
    /// The value for a String key.
    GetResponse(String),
    /// If no value exists for a key.
    NotFound,
    /// Set key to value.
    Set(String, String),
    /// Successful set response.
    SetResponse,
    /// Remove the entry for the given key.
    Remove(String),
    /// Successful remove response.
    RemoveResponse,
    /// An error from the server
    Error(String),
}

const GET_TAG: u8 = 0;
const GET_RESPONSE_TAG: u8 = 1;
const NOT_FOUND_TAG: u8 = 2;
const SET_TAG: u8 = 3;
const SET_RESPONSE_TAG: u8 = 4;
const REMOVE_TAG: u8 = 5;
const REMOVE_RESPONSE_TAG: u8 = 6;
const ERROR_TAG: u8 = 7;

impl Command {
    /// Serializes the command into bytes.
    pub fn serialize(&self) -> Vec<u8> {
        // First 4 bytes are the size of the message.
        let mut bytes = vec![0u8, 0u8, 0u8, 0u8];

        // The 5th byte is the tag for the type of command.
        match self {
            Command::Get(_) => bytes.push(GET_TAG),
            Command::GetResponse(_) => bytes.push(GET_RESPONSE_TAG),
            Command::NotFound => bytes.push(NOT_FOUND_TAG),
            Command::Set(_, _) => bytes.push(SET_TAG),
            Command::SetResponse => bytes.push(SET_RESPONSE_TAG),
            Command::Remove(_) => bytes.push(REMOVE_TAG),
            Command::RemoveResponse => bytes.push(REMOVE_RESPONSE_TAG),
            Command::Error(_) => bytes.push(ERROR_TAG),
        }

        // The remaining bytes are for each method.
        match self {
            Command::Get(key) => {
                extend_string(&mut bytes, key);
            }
            Command::GetResponse(value) => {
                extend_string(&mut bytes, value);
            }
            Command::NotFound => {}
            Command::Set(key, value) => {
                extend_string(&mut bytes, key);
                extend_string(&mut bytes, value);
            }
            Command::SetResponse => {}
            Command::Remove(key) => {
                extend_string(&mut bytes, key);
            }
            Command::RemoveResponse => {}
            Command::Error(msg) => {
                extend_string(&mut bytes, msg);
            }
        }
        // Overwrite size in first 4 bytes.
        let len = bytes.len() as u32;
        for (i, byte) in len.to_le_bytes().iter().enumerate() {
            bytes[i] = *byte
        }
        bytes
    }

    /// Deserializes bytes into a command.
    pub fn deserialize(bytes: &[u8]) -> Result<Command, String> {
        eprintln!("bytes: {:?}", bytes);
        assert!(bytes.len() > 4);
        // Size is used by stream parser to allocate the byte buffer.
        let _size = read_u32(bytes, 0);
        let cmd = match bytes[4] {
            GET_TAG => {
                let key_size = read_u32(bytes, 5) as usize;
                let key = read_string(bytes, 9, key_size);
                Command::Get(String::from(key))
            }
            GET_RESPONSE_TAG => {
                let value_size = read_u32(bytes, 5) as usize;
                let value = read_string(bytes, 9, value_size);
                Command::GetResponse(String::from(value))
            }
            NOT_FOUND_TAG => Command::NotFound,
            SET_TAG => {
                let key_size = read_u32(bytes, 5) as usize;
                let key = read_string(bytes, 9, key_size);
                let value_size = read_u32(bytes, 9 + key_size) as usize;
                let value = read_string(bytes, 4 + 9 + key_size, value_size);
                Command::Set(String::from(key), String::from(value))
            }
            SET_RESPONSE_TAG => Command::SetResponse,
            REMOVE_TAG => {
                let key_size = read_u32(bytes, 5) as usize;
                let key = read_string(bytes, 9, key_size);
                Command::Remove(String::from(key))
            }
            REMOVE_RESPONSE_TAG => Command::RemoveResponse,
            ERROR_TAG => {
                let msg_size = read_u32(bytes, 5) as usize;
                let msg = read_string(bytes, 9, msg_size);
                Command::Error(String::from(msg))
            }
            _ => panic!(),
        };
        Ok(cmd)
    }
}

fn read_u32(bytes: &[u8], lo: usize) -> u32 {
    u32::from_le_bytes([bytes[lo], bytes[lo + 1], bytes[lo + 2], bytes[lo + 3]])
}

fn read_string(bytes: &[u8], offset: usize, len: usize) -> &str {
    str::from_utf8(&bytes[offset..offset + len]).unwrap()
}

fn extend_string(v: &mut Vec<u8>, s: &str) {
    let len = s.bytes().len() as u32;
    v.extend(&len.to_le_bytes());
    v.extend(s.bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_get() {
        let key = "foo";
        let actual = Command::Get(String::from(key)).serialize();
        let mut expected = vec![12, 0, 0, 0, GET_TAG];
        extend_string(&mut expected, key);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_get_response() {
        let value = "foo";
        let actual = Command::GetResponse(String::from(value)).serialize();
        let mut expected = vec![12, 0, 0, 0, GET_RESPONSE_TAG];
        extend_string(&mut expected, value);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_not_found() {
        let actual = Command::NotFound.serialize();
        let mut expected = vec![5, 0, 0, 0, NOT_FOUND_TAG];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_set() {
        let key = "foo";
        let value = "bar";
        let actual = Command::Set(String::from(key), String::from(value)).serialize();
        let mut expected = vec![19, 0, 0, 0, SET_TAG];
        extend_string(&mut expected, key);
        extend_string(&mut expected, value);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_set_response() {
        let actual = Command::SetResponse.serialize();
        let mut expected = vec![5, 0, 0, 0, SET_RESPONSE_TAG];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_remove() {
        let key = "foo";
        let actual = Command::Remove(String::from(key)).serialize();
        let mut expected = vec![12, 0, 0, 0, REMOVE_TAG];
        extend_string(&mut expected, key);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_remove_response() {
        let actual = Command::RemoveResponse.serialize();
        let mut expected = vec![5, 0, 0, 0, REMOVE_RESPONSE_TAG];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_error() {
        let msg = "foo";
        let actual = Command::Error(String::from(msg)).serialize();
        let mut expected = vec![12, 0, 0, 0, ERROR_TAG];
        extend_string(&mut expected, msg);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deserialize_get() {
        let cmd = Command::Get(String::from("foo"));
        let actual = Command::deserialize(&cmd.serialize()).unwrap();
        assert_eq!(actual, cmd);
    }

    #[test]
    fn test_deserialize_get_response() {
        let cmd = Command::GetResponse(String::from("foo"));
        let actual = Command::deserialize(&cmd.serialize()).unwrap();
        assert_eq!(actual, cmd);
    }

    #[test]
    fn test_deserialize_not_found() {
        let cmd = Command::NotFound;
        let actual = Command::deserialize(&cmd.serialize()).unwrap();
        assert_eq!(actual, cmd);
    }

    #[test]
    fn test_deserialize_set() {
        let cmd = Command::Set(String::from("key"), String::from("bar"));
        let actual = Command::deserialize(&cmd.serialize()).unwrap();
        assert_eq!(actual, cmd);
    }

    #[test]
    fn test_deserialize_set_response() {
        let cmd = Command::SetResponse;
        let actual = Command::deserialize(&cmd.serialize()).unwrap();
        assert_eq!(actual, cmd);
    }

    #[test]
    fn test_deserialize_remove() {
        let key = "foo";
        let cmd = Command::Remove(String::from("foo"));
        let actual = Command::deserialize(&cmd.serialize()).unwrap();
        assert_eq!(actual, cmd);
    }

    #[test]
    fn test_deserialize_remove_response() {
        let cmd = Command::RemoveResponse;
        let actual = Command::deserialize(&cmd.serialize()).unwrap();
        assert_eq!(actual, cmd);
    }

    #[test]
    fn test_deserialize_error() {
        let key = "foo";
        let cmd = Command::Error(String::from(key));
        let actual = Command::deserialize(&cmd.serialize()).unwrap();
        assert_eq!(actual, cmd);
    }
}
