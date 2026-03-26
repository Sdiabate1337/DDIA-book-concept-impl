//! Integration tests: spin up a real TCP server in the background and talk
//! to it using `TcpStream`, exercising the full request/response cycle.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

/// Bind the server to an OS-assigned port and return the address string.
fn start_server() -> String {
    // Import needed types from the crate.
    use memkv::server::Server;

    let srv = Server::new("127.0.0.1:0").expect("bind failed");
    let addr = srv.local_addr().expect("local_addr").to_string();

    thread::spawn(move || srv.run());
    // Give the server a moment to start accepting.
    thread::sleep(Duration::from_millis(50));
    addr
}

fn send(stream: &mut TcpStream, reader: &mut BufReader<TcpStream>, cmd: &str) -> String {
    stream
        .write_all(format!("{}\n", cmd).as_bytes())
        .expect("write");
    stream.flush().expect("flush");
    let mut line = String::new();
    reader.read_line(&mut line).expect("read_line");
    line
}

#[test]
fn ping_pong() {
    let addr = start_server();
    let stream = TcpStream::connect(&addr).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream;

    let resp = send(&mut writer, &mut reader, "PING");
    assert_eq!(resp.trim(), "+PONG");
}

#[test]
fn set_get_del() {
    let addr = start_server();
    let stream = TcpStream::connect(&addr).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream;

    assert_eq!(send(&mut writer, &mut reader, "SET name rust").trim(), "+OK");
    assert_eq!(send(&mut writer, &mut reader, "GET name").trim(), "$rust");
    assert_eq!(send(&mut writer, &mut reader, "EXISTS name").trim(), ":1");
    assert_eq!(send(&mut writer, &mut reader, "DEL name").trim(), ":1");
    assert_eq!(send(&mut writer, &mut reader, "GET name").trim(), "$NIL");
    assert_eq!(send(&mut writer, &mut reader, "EXISTS name").trim(), ":0");
}

#[test]
fn get_missing_key_returns_nil() {
    let addr = start_server();
    let stream = TcpStream::connect(&addr).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream;

    let resp = send(&mut writer, &mut reader, "GET no_such_key");
    assert_eq!(resp.trim(), "$NIL");
}

#[test]
fn unknown_command_returns_error() {
    let addr = start_server();
    let stream = TcpStream::connect(&addr).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream;

    let resp = send(&mut writer, &mut reader, "FLUSHALL");
    assert!(resp.starts_with("-ERR"), "expected error, got: {}", resp);
}

#[test]
fn set_value_with_spaces() {
    let addr = start_server();
    let stream = TcpStream::connect(&addr).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream;

    send(&mut writer, &mut reader, "SET msg hello world");
    let resp = send(&mut writer, &mut reader, "GET msg");
    assert_eq!(resp.trim(), "$hello world");
}

#[test]
fn multiple_clients_isolated() {
    let addr = start_server();

    let mut handles = vec![];
    for i in 0..5u32 {
        let addr = addr.clone();
        handles.push(thread::spawn(move || {
            let stream = TcpStream::connect(&addr).unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = stream;

            let key = format!("client{}", i);
            let val = format!("value{}", i);
            send(&mut writer, &mut reader, &format!("SET {} {}", key, val));
            let got = send(&mut writer, &mut reader, &format!("GET {}", key));
            assert_eq!(got.trim(), format!("${}", val));
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}
