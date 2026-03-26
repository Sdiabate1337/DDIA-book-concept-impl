//! Wire protocol for memkv.
//!
//! # Request format
//!
//! Each request is a single UTF-8 line terminated by `\r\n` (or just `\n`).
//! Tokens are separated by ASCII space.
//!
//! ```text
//! PING\r\n
//! SET  <key> <value>\r\n
//! GET  <key>\r\n
//! DEL  <key>\r\n
//! EXISTS <key>\r\n
//! ```
//!
//! # Response format
//!
//! | Prefix | Meaning              |
//! |--------|----------------------|
//! | `+`    | Simple string        |
//! | `-`    | Error                |
//! | `:`    | Integer              |
//! | `$`    | Bulk string (value)  |
//! | `$NIL` | Null / key not found |

/// A parsed client command.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Command {
    Ping,
    Set { key: String, value: String },
    Get { key: String },
    Del { key: String },
    Exists { key: String },
}

/// A server response.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Response {
    Pong,
    Ok,
    Value(String),
    Nil,
    Integer(i64),
    Error(String),
}

impl Response {
    /// Serialise the response to the wire format.
    pub fn encode(&self) -> String {
        match self {
            Response::Pong => "+PONG\r\n".into(),
            Response::Ok => "+OK\r\n".into(),
            Response::Value(v) => format!("${}\r\n", v),
            Response::Nil => "$NIL\r\n".into(),
            Response::Integer(n) => format!(":{}\r\n", n),
            Response::Error(msg) => format!("-ERR {}\r\n", msg),
        }
    }
}

/// Parse a raw line (without the trailing newline) into a `Command`.
///
/// Returns an error string suitable for sending back to the client when the
/// input is malformed.
pub fn parse(line: &str) -> Result<Command, String> {
    let line = line.trim();
    let mut parts = line.splitn(3, ' ');

    let verb = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "empty command".to_string())?
        .to_uppercase();

    match verb.as_str() {
        "PING" => Ok(Command::Ping),
        "SET" => {
            let key = parts
                .next()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "SET requires a key".to_string())?
                .to_string();
            let value = parts
                .next()
                .ok_or_else(|| "SET requires a value".to_string())?
                .to_string();
            Ok(Command::Set { key, value })
        }
        "GET" => {
            let key = parts
                .next()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "GET requires a key".to_string())?
                .to_string();
            Ok(Command::Get { key })
        }
        "DEL" => {
            let key = parts
                .next()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "DEL requires a key".to_string())?
                .to_string();
            Ok(Command::Del { key })
        }
        "EXISTS" => {
            let key = parts
                .next()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "EXISTS requires a key".to_string())?
                .to_string();
            Ok(Command::Exists { key })
        }
        other => Err(format!("unknown command '{}'", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ping() {
        assert_eq!(parse("PING"), Ok(Command::Ping));
        assert_eq!(parse("ping"), Ok(Command::Ping));
    }

    #[test]
    fn parse_set() {
        assert_eq!(
            parse("SET foo bar"),
            Ok(Command::Set {
                key: "foo".into(),
                value: "bar".into(),
            })
        );
    }

    #[test]
    fn parse_set_value_with_spaces() {
        assert_eq!(
            parse("SET greeting hello world"),
            Ok(Command::Set {
                key: "greeting".into(),
                value: "hello world".into(),
            })
        );
    }

    #[test]
    fn parse_get() {
        assert_eq!(parse("GET foo"), Ok(Command::Get { key: "foo".into() }));
    }

    #[test]
    fn parse_del() {
        assert_eq!(parse("DEL foo"), Ok(Command::Del { key: "foo".into() }));
    }

    #[test]
    fn parse_exists() {
        assert_eq!(
            parse("EXISTS foo"),
            Ok(Command::Exists { key: "foo".into() })
        );
    }

    #[test]
    fn parse_empty_returns_error() {
        assert!(parse("").is_err());
        assert!(parse("   ").is_err());
    }

    #[test]
    fn parse_unknown_command() {
        assert!(parse("FLUSHALL").is_err());
    }

    #[test]
    fn parse_set_missing_value() {
        assert!(parse("SET key").is_err());
    }

    #[test]
    fn parse_get_missing_key() {
        assert!(parse("GET").is_err());
    }

    #[test]
    fn response_encode_pong() {
        assert_eq!(Response::Pong.encode(), "+PONG\r\n");
    }

    #[test]
    fn response_encode_ok() {
        assert_eq!(Response::Ok.encode(), "+OK\r\n");
    }

    #[test]
    fn response_encode_value() {
        assert_eq!(Response::Value("hello".into()).encode(), "$hello\r\n");
    }

    #[test]
    fn response_encode_nil() {
        assert_eq!(Response::Nil.encode(), "$NIL\r\n");
    }

    #[test]
    fn response_encode_integer() {
        assert_eq!(Response::Integer(1).encode(), ":1\r\n");
        assert_eq!(Response::Integer(0).encode(), ":0\r\n");
    }

    #[test]
    fn response_encode_error() {
        assert_eq!(
            Response::Error("oops".into()).encode(),
            "-ERR oops\r\n"
        );
    }
}
