//! TCP server that accepts client connections and dispatches commands.
//!
//! Each accepted connection is handled in its own OS thread.  The shared
//! `Store` is cheap to clone (it wraps an `Arc`) so every connection thread
//! gets its own handle without any additional allocation.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread;

use crate::protocol::{self, Command, Response};
use crate::store::Store;

/// A running TCP server.
pub struct Server {
    listener: TcpListener,
    store: Store,
}

impl Server {
    /// Bind to `addr` and create a new `Server`.
    pub fn new<A: ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(Server {
            listener,
            store: Store::new(),
        })
    }

    /// Return the local address the server is listening on.
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    /// Accept connections and process them forever (blocking).
    pub fn run(self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let store = self.store.clone();
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream, store) {
                            eprintln!("connection error: {}", e);
                        }
                    });
                }
                Err(e) => eprintln!("accept error: {}", e),
            }
        }
    }
}

/// Handle a single client connection: read lines, execute commands, write responses.
fn handle_connection(stream: TcpStream, store: Store) -> std::io::Result<()> {
    let mut writer = stream.try_clone()?;
    let reader = BufReader::new(stream);

    for line in reader.lines() {
        let line = line?;
        let response = match protocol::parse(&line) {
            Err(msg) => Response::Error(msg),
            Ok(cmd) => execute(cmd, &store),
        };
        writer.write_all(response.encode().as_bytes())?;
    }
    Ok(())
}

/// Execute a parsed command against `store` and return the appropriate response.
pub fn execute(cmd: Command, store: &Store) -> Response {
    match cmd {
        Command::Ping => Response::Pong,
        Command::Set { key, value } => {
            store.set(key, value);
            Response::Ok
        }
        Command::Get { key } => match store.get(&key) {
            Some(v) => Response::Value(v),
            None => Response::Nil,
        },
        Command::Del { key } => {
            let removed = store.del(&key);
            Response::Integer(if removed { 1 } else { 0 })
        }
        Command::Exists { key } => {
            let present = store.exists(&key);
            Response::Integer(if present { 1 } else { 0 })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn make_store() -> Store {
        Store::new()
    }

    #[test]
    fn execute_ping() {
        let store = make_store();
        assert_eq!(execute(Command::Ping, &store), Response::Pong);
    }

    #[test]
    fn execute_set_get() {
        let store = make_store();
        assert_eq!(
            execute(
                Command::Set {
                    key: "k".into(),
                    value: "v".into()
                },
                &store
            ),
            Response::Ok
        );
        assert_eq!(
            execute(Command::Get { key: "k".into() }, &store),
            Response::Value("v".into())
        );
    }

    #[test]
    fn execute_get_missing() {
        let store = make_store();
        assert_eq!(
            execute(Command::Get { key: "missing".into() }, &store),
            Response::Nil
        );
    }

    #[test]
    fn execute_del_existing() {
        let store = make_store();
        store.set("k".into(), "v".into());
        assert_eq!(
            execute(Command::Del { key: "k".into() }, &store),
            Response::Integer(1)
        );
    }

    #[test]
    fn execute_del_missing() {
        let store = make_store();
        assert_eq!(
            execute(Command::Del { key: "nope".into() }, &store),
            Response::Integer(0)
        );
    }

    #[test]
    fn execute_exists() {
        let store = make_store();
        assert_eq!(
            execute(Command::Exists { key: "k".into() }, &store),
            Response::Integer(0)
        );
        store.set("k".into(), "v".into());
        assert_eq!(
            execute(Command::Exists { key: "k".into() }, &store),
            Response::Integer(1)
        );
    }
}
