#![allow(dead_code)]
extern crate mio;

mod net;
mod conn;
mod tcp;

pub use self::net::{Network, ConnectionsMap, NetworkCommand, NetworkCMD};
pub use self::conn::{Connection, ConnectionIdentity, SocketType};
use std::u32::MAX as u32MAX;
use self::mio::Token;

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

pub const RECEIVER_CHANNEL_TOKEN: Token = Token((u32MAX - 1) as usize);
pub const LOOP_EVENTS_COUNT: usize = 64000;