#![allow(dead_code)]
extern crate mio;

use helper::Log;

use self::mio::tcp::TcpListener;
use self::mio::{Ready, PollOpt};
use node::{Node, NET_TCP_SERVER_TOKEN};

use std::net::SocketAddr;
use std::error::Error;
use std::process;
use std::str::FromStr;

/// TcpNetwork Trait for implementing TCP networking capabilities
/// On top of Node structure
pub trait TcpNetwork {
    /// Min function to attach TCP service functionality to existing POLL service
    fn register_tcp(&mut self);

    /// Make TCP server socket listener from given address
    fn make_tcp_server(address: &str) -> TcpListener;
}

impl TcpNetwork for Node {
    fn register_tcp(&mut self) {
        match self.poll.register(&self.net_tcp_server, NET_TCP_SERVER_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {}
            Err(e) => {
                Log::error("Unable to register TCP server to Node POLL service", e.description());
                process::exit(1);
            }
        }
    }

    fn make_tcp_server(address: &str) -> TcpListener {
        let addr = match SocketAddr::from_str(address) {
            Ok(a) => a,
            Err(e) => {
                Log::error("Unable to parse given TCP server address", e.description());
                process::exit(1);
            }
        };

        match TcpListener::bind(&addr) {
            Ok(s) => s,
            Err(e) => {
                Log::error("Unable to bind given TCP server address", e.description());
                process::exit(1);
            }
        }
    }
}

