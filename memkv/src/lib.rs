//! memkv — distributed in-memory key-value store.
//!
//! # Architecture
//!
//! - [`store::Store`] — thread-safe `HashMap` wrapped in `Arc<RwLock<_>>`
//! - [`protocol`]     — parse/encode the line-based wire protocol
//! - [`server::Server`] — multi-threaded TCP server (one thread per connection)

pub mod store;
pub mod protocol;
pub mod server;
