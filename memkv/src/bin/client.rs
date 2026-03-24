//! Interactive command-line client for memkv.
//!
//! Usage:
//!   memkv-client [host:port]
//!
//! Defaults to 127.0.0.1:6379 when no address is given.
//! Type any supported command (PING, SET, GET, DEL, EXISTS) and press Enter.
//! Type `quit` or `exit` to disconnect.

use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

fn main() -> io::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6379".to_string());

    let stream = TcpStream::connect(&addr)?;
    println!("Connected to memkv at {}. Type PING, SET <k> <v>, GET <k>, DEL <k>, EXISTS <k>.", addr);

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line?;
        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit") {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        writer.write_all(format!("{}\n", trimmed).as_bytes())?;
        writer.flush()?;

        let mut response = String::new();
        reader.read_line(&mut response)?;
        print!("{}", response);
    }

    Ok(())
}
