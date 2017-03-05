#![allow(dead_code)]
extern crate slab;
extern crate mio;

use self::mio::{Token, Poll, Ready, PollOpt};
use self::mio::tcp::{TcpListener};
use network::tcp::{TcpReaderConn};
use network::{Network, ConnectionsMap};
use std::error::Error;
use logger::Log;
use std::process;
use std::net::SocketAddr;
use std::str::FromStr;
use std::u32::MAX as u32MAX;

const SERVER_SOCKET_TOKEN: Token = Token((u32MAX - 2) as usize);
type Slab<T> = slab::Slab<T, Token>;
const CONNECTION_COUNT_PRE_ALLOC: usize = 1024;

// Main struct to handle TCP networking
pub struct TcpNetwork {
    // pending tcp connections, which just accepted but not sent API version and Prime Value
    pending_connections: Slab<TcpReaderConn>,

    // server socket for TCP listener
    server_socket: TcpListener,
}

impl TcpNetwork {
    pub fn new(server_address: &str) -> TcpNetwork {
        // making TcpListener for making server socket
        let addr = match SocketAddr::from_str(server_address) {
            Ok(a) => a,
            Err(e) => {
                Log::error(format!("Unable to parse given server address {}", server_address).as_str(), e.description());
                process::exit(1);
            }
        };

        // binding TCP server
        let server_socket = match TcpListener::bind(&addr) {
            Ok(s) => s,
            Err(e) => {
                Log::error(format!("Unable to bind TCP Server to given address {}", server_address).as_str(), e.description());
                process::exit(1);
            }
        };

        TcpNetwork {
            pending_connections: Slab::with_capacity(CONNECTION_COUNT_PRE_ALLOC),
            server_socket: server_socket
        }
    }

    pub fn register(&self, poll: &mut Poll) {
        match poll.register(&self.server_socket, SERVER_SOCKET_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                Log::error("Error while tryig to register TCP server to main networking loop", e.description());
                process::exit(1);
            }
        };
    }

    #[inline(always)]
    pub fn ready(&self, token: Token, poll: &mut Poll, conns: &mut ConnectionsMap) -> bool {
        false
    }
}
