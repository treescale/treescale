#![allow(dead_code)]
mod net;
mod conn;
mod tcp;

pub use self::net::{Network, ConnectionsMap};
pub use self::conn::Connection;

// main configuration for Networking
pub struct NetworkConfig {
    server_address: String,
    concurrency: usize,
}
