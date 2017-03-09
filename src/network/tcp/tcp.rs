#![allow(dead_code)]
extern crate mio;

use self::mio::{Token, Poll, Ready, PollOpt};
use self::mio::channel::Sender;
use self::mio::tcp::{TcpListener, TcpStream};
use network::tcp::{TcpReaderConn, Slab, TcpReader, TcpReaderCommand, TcpWriter, TcpWriterCommand};
use network::{ConnectionsMap, Connection};
use std::error::Error;
use logger::Log;
use std::process;
use std::net::SocketAddr;
use std::str::FromStr;
use std::u32::MAX as u32MAX;

const SERVER_SOCKET_TOKEN: Token = Token((u32MAX - 2) as usize);
const CONNECTION_COUNT_PRE_ALLOC: usize = 1024;

// Main struct to handle TCP networking
pub struct TcpNetwork {
    // pending tcp connections, which just accepted but not sent API version and Prime Value
    pending_connections: Slab<TcpReaderConn>,

    // server socket for TCP listener
    server_socket: TcpListener,

    // list of channels for TCP reader/writer
    reader_channels: Vec<Sender<TcpReaderCommand>>,
    writer_channels: Vec<Sender<TcpWriterCommand>>,
    rw_index: usize
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
            server_socket: server_socket,
            reader_channels: vec![],
            writer_channels: vec![],
            rw_index: 0
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
        let (mut close_conn, token_value) = {
            let ref mut conn = self.pending_connections[token];
            if !Connection::check_api_version(conn.api_version) {
                match conn.read_api_version() {
                    Some((done, version)) => {
                        // if we done reading API version
                        if done {
                            if !Connection::check_api_version(version) {
                                Log::warn("API version of TCP connection is wrong", version.to_string().as_str());
                                (true, None) // If API version is wrong, close connection
                            } else {
                                // setting API version for later usage
                                conn.api_version = version;
                                (false, None)
                            }
                        } else {
                            // if there is still data, just returning and waiting more data
                            (false, None)
                        }
                    }
                    None => (true, None) // Close connection
                }
            } else {
                match conn.read_token_value() {
                    Some((done, conn_token, value)) => {
                        // if we got here then we got Token and Value Handshake
                        // Accepting connection and moving it to one of the readers/writers
                    
                        if done {
                            (true, Some((conn_token, value)))
                        } else {
                            (false, None)
                        }
                    }

                    None => (true, None) // Close connection
                }
            }
        };

        if !token_value.is_none() {
            let (token_str, value) = token_value.unwrap();
            let writer_conn = self.pending_connections[token].make_writer();
            // if we can't create writer connection
            // just closing accepted connection
            if writer_conn.is_none() {
                close_conn = true
            } else {
                let (index, reader, writer) = self.get_read_writer();
                let mut reader_conn = self.pending_connections.remove(token).unwrap();
                let mut writer_conn = writer_conn.unwrap();
                reader_conn.conn_token = token_str.clone();
                writer_conn.conn_token = token_str.clone();
                Implement Reader Writer Commands
                let _ = reader.send(TcpReaderCommand {});
                let _ = writer.send(TcpWriterCommand {});
                conns.insert(token_str.clone()
                , Connection::new(
                    token_str.clone(), value, index, reader_conn.from_server
                ));
            }
        }

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
        match self.pending_connections.remove(token) {
            Some(conn) => {
                    conn.close();
                    // dropping connection object from memory and closing it
                    drop(conn);
                },
            None => {}
        }
    }

    #[inline(always)]
    fn get_read_writer(&mut self) -> (usize, Sender<TcpReaderCommand>, Sender<TcpWriterCommand>) {
        if self.rw_index >= self.reader_channels.len() {
            self.rw_index = 0;
        }
        let i = self.rw_index;
        self.rw_index += 1;
        (i, self.reader_channels[i].clone(), self.writer_channels[i].clone())
    }
}
