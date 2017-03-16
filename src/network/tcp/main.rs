#![allow(dead_code)]
extern crate mio;

use helper::Log;

use self::mio::tcp::TcpListener;
use self::mio::{Ready, PollOpt, Token};
use self::mio::channel::Sender;

use node::{Node, NET_TCP_SERVER_TOKEN};
use network::{CONNECTION_COUNT_PRE_ALLOC, TcpConnection
              , Connection, Networking
              , TcpHandlerCommand, TcpHandlerCMD};
use helper::NetHelper;

use std::net::SocketAddr;
use std::error::Error;
use std::process;
use std::str::FromStr;
use std::io::ErrorKind;
use std::sync::Arc;

/// TcpNetwork Trait for implementing TCP networking capabilities
/// On top of Node structure
pub trait TcpNetwork {
    /// Min function to attach TCP service functionality to existing POLL service
    fn register_tcp(&mut self);

    /// Make TCP server socket listener from given address
    fn make_tcp_server(address: &str) -> TcpListener;

    /// Handler for event loop ready event
    /// This is general event processing for TCP connections/servers
    /// If event token not in the TCP list it will return false
    /// To let other components to handle event
    fn tcp_ready(&mut self, token: Token, event_kind: Ready) -> bool;

    /// Function for accepting TCP connections as a pending connections
    fn tcp_acceptable(&mut self);

    /// Function for reading TCP data from pending connections
    fn tcp_readable(&mut self, token: Token);
    /// Function for writing TCP data from pending connections
    fn tcp_writable(&mut self, token: Token);
    /// closing TCP connection
    fn tcp_close(&mut self, token: Token);
    /// getting one of the TCP handler channels
    /// using Round Rubin algorithm
    fn tcp_get_handler(&mut self) -> Sender<TcpHandlerCommand>;
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

    #[inline(always)]
    fn tcp_ready(&mut self, token: Token, event_kind: Ready) -> bool {
        if token == NET_TCP_SERVER_TOKEN {
            if event_kind != Ready::readable() {
                Log::error("Unexpected TCP Server event kind", "Ignoring for now!");
                return false;
            }

            self.tcp_acceptable();
            return true;
        }

        if self.net_tcp_pending_connections.contains(token) {
            if event_kind == Ready::readable() {

                self.tcp_readable(token);

            } else if event_kind == Ready::writable() {

                self.tcp_writable(token);

            } else if event_kind == Ready::error()
                || event_kind == Ready::hup() {

                self.tcp_close(token);

            }

            return true;
        }

        false
    }

    #[inline(always)]
    fn tcp_acceptable(&mut self) {
        loop {
            let sock = match self.net_tcp_server.accept() {
                Ok((s, _)) => s,
                Err(e) => {
                    // if we got WouldBlock, then this is Non Blocking socket
                    // and data still not available for this, so it's not a connection error
                    if e.kind() != ErrorKind::WouldBlock {
                        Log::error("Unable to accept connection from TCP server socket", e.description());
                    }
                    return;
                }
            };

            // inserting connection to pending connections and registering to the loop
            if self.net_tcp_pending_connections.vacant_entry().is_none() {
                self.net_tcp_pending_connections.reserve_exact(CONNECTION_COUNT_PRE_ALLOC);
            }

            let entry = match self.net_tcp_pending_connections.vacant_entry() {
                Some(e) => e,
                None => {
                    Log::error("Unable to insert accepted connection to TcpNetwork pending connections"
                               , "Not enough place in Slab");
                    return;
                }
            };

            // creating connection and registering to current Node poll service
            let conn = TcpConnection::new(sock, entry.index(), true);
            conn.register(&self.poll);
            entry.insert(conn);
        };
    }

    #[inline(always)]
    fn tcp_readable(&mut self, token: Token) {
        // if we got here then we have connection with this token
        let mut close_conn = {
            let ref mut conn: TcpConnection = self.net_tcp_pending_connections[token];
            // if we don't have yet API version defined
            if !Connection::check_api_version(conn.api_version) {
                match conn.read_api_version() {
                    Some((done, version)) => {
                        // if we not done with reading API version
                        // Just returning and waiting until next readable cycle
                        if !done {
                            return;
                        }

                        // if we got wrong API version just closing connection
                        if !Connection::check_api_version(version) {
                            true
                        } else {
                            // if we got valid API version
                            // saving it as a connection version
                            conn.api_version = version;
                            false
                        }
                    }

                    // if we have connection error closing it
                    None => true
                }
            } else {
                false
            }
        };

        if close_conn {
            self.tcp_close(token);
            return;
        }

        close_conn = {
            let ref mut conn: TcpConnection = self.net_tcp_pending_connections[token];
            // reading Connection Token and Value
            match conn.read_token_value() {
                Some((done, token_str, value)) => {
                    // if we not done with reading API version
                    // Just returning and waiting until next readable cycle
                    if !done {
                        return;
                    }

                    // checking if we got valid Prime Value or not
                    // if it's invalid just closing connection
                    if !NetHelper::validate_value(value) {
                        true
                    } else {
                        // if we done with token and value
                        // just setting them for connection
                        // and writing API handshake information
                        conn.conn_token = token_str;
                        conn.conn_value = value;

                        false
                    }
                }

                // if we have connection error closing it
                None => true
            }
        };

        if close_conn {
            self.tcp_close(token);
            return;
        }

        // if we got here then we have connection information
        // so now we need to send our handshake information
        // and after write will succeed, we will transfer connection
        // to one of the TCP handlers
        let info = Arc::new(self.handshake_info());
        self.net_tcp_pending_connections[token].write(info, &self.poll);
    }

    #[inline(always)]
    fn tcp_writable(&mut self, token: Token) {
        // if we got here then we have connection with this token
        let close_conn = {
            let ref mut conn: TcpConnection = self.net_tcp_pending_connections[token];
            match conn.flush() {
                Some(done) => {
                    // if Write queue is not empty, just returning
                    // and waiting for the next cycle
                    if !done {
                        return;
                    }

                    // letting know to keep connection
                    // so that we can make sure that queue is empty
                    false
                }

                // closing connection if we have write error
                None => true
            }
        };

        if close_conn {
            self.tcp_close(token);
            return;
        }

        // if we got here then write is done
        // so moving connection to one of the TCP handlers
        let mut command = TcpHandlerCommand::new();
        // removing connection from pending connections list
        let conn = self.net_tcp_pending_connections.remove(token).unwrap();
        // de-registering from current event loop
        self.poll.deregister(&conn.socket);

        command.cmd = TcpHandlerCMD::HandleConnection;
        command.conn.push(conn);
        match self.tcp_get_handler().send(command) {
            Ok(_) => {},
            Err(e) => {
                Log::error("Unable to send HandleConnection command to TCP handler", e.description());
                return;
            }
        }
    }

    #[inline(always)]
    fn tcp_close(&mut self, token: Token) {
        // just removing connection
        // it would be closed automatically when
        // connection object de-allocated from memory
        self.net_tcp_pending_connections.remove(token);
    }

    #[inline(always)]
    fn tcp_get_handler(&mut self) -> Sender<TcpHandlerCommand> {
        if self.net_tcp_handler_index >= self.net_tcp_reader_sender_chan.len() {
            self.net_tcp_handler_index = 0;
        }

        let i = self.net_tcp_handler_index;
        self.net_tcp_handler_index += 1;

        self.net_tcp_reader_sender_chan[i].clone()
    }
}

