#![allow(dead_code)]
mod net;
mod conn;
mod tcp;

pub use self::net::{Network, ConnectionsMap, NetworkCommand, NetworkCMD};
pub use self::conn::Connection;

// main configuration for Networking
pub struct NetworkConfig {
    pub api_version: u32,
    pub server_address: String,
    pub concurrency: usize,
}


impl NetworkConfig {
    pub fn default() -> NetworkConfig {
        NetworkConfig {
            api_version: 0,
            server_address: String::new(),
            concurrency: 1
        }
    }
}