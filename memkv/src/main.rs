use std::env;

use memkv::server::Server;

fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6379".to_string());

    let srv = Server::new(&addr).expect("failed to bind");
    println!("memkv listening on {}", addr);
    srv.run();
}

