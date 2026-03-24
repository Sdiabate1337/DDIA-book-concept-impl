# memkv — Distributed In-Memory Key-Value Store

> A Rust implementation of core DDIA (Designing Data-Intensive Applications) concepts:
> concurrent data structures, wire protocols, and multi-client servers.

## Concepts illustrated

| DDIA concept | Where in this code |
|---|---|
| Thread-safe shared state | `Store` — `Arc<RwLock<HashMap>>` |
| Read/write concurrency | Multiple readers, single writer via `RwLock` |
| Client-server protocol | Line-based text protocol in `protocol.rs` |
| Network I/O | `TcpListener` + per-connection thread in `server.rs` |

## Project structure

```
memkv/
├── src/
│   ├── lib.rs          # public API
│   ├── main.rs         # server binary entry-point
│   ├── store.rs        # thread-safe key-value store
│   ├── protocol.rs     # parse / encode wire messages
│   ├── server.rs       # TCP server + command executor
│   └── bin/
│       └── client.rs   # interactive CLI client
└── tests/
    └── integration.rs  # end-to-end TCP tests
```

## Building

```bash
cargo build --release
```

## Running the server

```bash
# default address: 127.0.0.1:6379
cargo run --bin memkv-server

# custom address
cargo run --bin memkv-server -- 0.0.0.0:7000
```

## Using the interactive client

```bash
cargo run --bin memkv-client
# or with a custom address
cargo run --bin memkv-client -- 127.0.0.1:7000
```

### Supported commands

| Command | Description | Example |
|---|---|---|
| `PING` | Liveness check | `PING` → `+PONG` |
| `SET key value` | Store a value | `SET name rust` → `+OK` |
| `GET key` | Retrieve a value | `GET name` → `$rust` |
| `DEL key` | Delete a key | `DEL name` → `:1` |
| `EXISTS key` | Check existence | `EXISTS name` → `:0` |

### Response prefixes

| Prefix | Meaning |
|---|---|
| `+` | Simple string (e.g. `+OK`, `+PONG`) |
| `-` | Error (`-ERR <message>`) |
| `:` | Integer (e.g. `:1`, `:0`) |
| `$` | Bulk string value (e.g. `$rust`) |
| `$NIL` | Null — key not found |

## Running tests

```bash
cargo test
```
