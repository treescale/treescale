#![allow(dead_code)]
mod net;
mod conn;
mod tcp;

pub use self::net::{Network, ConnectionsMap, NetworkCommand, NetworkCMD};
pub use self::conn::Connection;

// main configuration for Networking
pub struct NetworkConfig {
    api_version: u32,
    server_address: String,
    concurrency: usize,
}
