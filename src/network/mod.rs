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


impl NetworkConfig {
    pub fn default() -> NetworkConfig {
        NetworkConfig {
            api_version: 0,
            server_address: String::from("0.0.0.0:8000"),
            concurrency: 1
        }
    }
}