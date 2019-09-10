use serde::ser::SerializeSeq;
use serde::Serializer;

#[derive(Debug)]
pub enum RedisType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RedisType>),
    Null,
}

impl serde::Serialize for RedisType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RedisType::SimpleString(s) => serializer.serialize_str(s),
            RedisType::Error(e) => serializer.serialize_str(e),
            RedisType::Integer(n) => serializer.serialize_i64(*n),
            RedisType::BulkString(s) => serializer.serialize_str(s),
            RedisType::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for redisType in v {
                    seq.serialize_element(redisType);
                }
                seq.end()
            }
            RedisType::Null => serializer.serialize_unit(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serde_resp;

    #[test]
    fn test_serialize_redis_type() {
        let actual = serde_resp::to_string(&RedisType::SimpleString("foo".to_string())).unwrap();
        assert_eq!(actual, "+foo\r\n");
    }
}
