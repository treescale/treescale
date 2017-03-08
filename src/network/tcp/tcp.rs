#![allow(dead_code)]
extern crate slab;
extern crate mio;

use self::mio::{Token, Poll, Ready, PollOpt};
use self::mio::tcp::{TcpListener, TcpStream};
use network::tcp::{TcpReaderConn};
use network::{ConnectionsMap, Connection};
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

    /// Callback for Networking event loop
    /// Will return true if Event Token is inside this TCP class
    /// Otherwise it will return false to give this event to other handlers
    #[inline(always)]
    pub fn ready(&mut self, token: Token, kind: Ready, poll: &mut Poll, conns: &mut ConnectionsMap) -> bool {
        if token == SERVER_SOCKET_TOKEN {
            self.acceptable(poll);
            return true
        } else if self.pending_connections.contains(token) {
            if kind == Ready::readable() {
                self.readable(token, poll, conns);
            }
            if kind == Ready::writable() {
                self.writable(token, poll, conns);
            }
            return true
        }

        false
    }

    // this function will insert accepted TCP socket as a connection to pending connections list
    #[inline(always)]
    fn insert_conn(&mut self, socket: TcpStream, from_server: bool, poll: &mut Poll) {
        // if we don't have a space in our connections array, just allocating more space
        if self.pending_connections.vacant_entry().is_none() {
            self.pending_connections.reserve_exact(CONNECTION_COUNT_PRE_ALLOC);
        }

        let entry = self.pending_connections.vacant_entry().unwrap();
        let token = entry.index();
        // registering connection to networking loop
        match poll.register(&socket, token, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                Log::error("Unable to register TCP accepted connection to Networking POLL", e.description());
                return;
            }
        };

        // inserting connection to our pending connections list
        entry.insert(TcpReaderConn::new(socket, token, true));
    }

    /// Main function for accepting TCP connections in Networking loop
    #[inline(always)]
    fn acceptable(&mut self, poll: &mut Poll) {
        loop {  
            let sock = match self.server_socket.accept() {
                Ok((s, _)) => s,
                Err(e) => {
                    Log::error("Unable to accept connection from TCP server socket", e.description());
                    return;
                }
            };

            // inserting connection to pending connections and registering to the loop
            self.insert_conn(sock, true, poll);
        };
    }

    /// Main function for reading data from connections, if there is something to read
    #[inline(always)]
    fn readable(&mut self, token: Token, poll: &mut Poll, conns: &mut ConnectionsMap) {
        let close_conn = {
            let ref mut conn = self.pending_connections[token];
            if !Connection::check_api_version(conn.version()) {
                match conn.read_api_version() {
                    Some((done, version)) => {
                        // if we done reading API version
                        if done {
                            if !Connection::check_api_version(version) {
                                Log::warn("API version of TCP connection is wrong", version.to_string().as_str());
                                true // If API version is wrong, close connection
                            } else {
                                // setting API version for later usage
                                conn.set_version(version);
                                false
                            }
                        } else {
                            // if there is still data, just returning and waiting more data
                            false
                        }
                    }
                    None => true // Close connection
                }
            } else {
                match conn.read_token_value() {
                    Some((done, conn_token, value)) => {
                        unimplemented!()
                        // false
                    }

                    None => true // Close connection
                }
            }
        };

        if close_conn {
            self.close_connection(token);
        }
    }

    /// Main function for writing to pending connections, while they are in main networking loop
    #[inline(always)]
    fn writable(&mut self, token: Token, poll: &mut Poll, conns: &mut ConnectionsMap) {
        let ref mut conn = self.pending_connections[token];
    }

    #[inline(always)]
    fn close_connection(&mut self, token: Token) {
        unimplemented!()
    }
}
