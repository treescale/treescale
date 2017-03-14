#![allow(dead_code)]
extern crate mio;
extern crate threadpool;

use self::mio::{Token, Poll, Ready, PollOpt};
use self::mio::channel::Sender;
use self::mio::tcp::{TcpListener, TcpStream};
use network::tcp::{TcpReaderConn, Slab
                    , TcpReader, TcpReaderCommand, TcpReaderCMD
                    , TcpWriter, TcpWriterCommand, TcpWriterCMD
                    , CONNECTION_COUNT_PRE_ALLOC, SERVER_SOCKET_TOKEN};
use network::{ConnectionsMap, Connection, ConnectionIdentity, SocketType, NetworkCommand};
use std::error::Error;
use helper::Log;
use std::process;
use std::net::SocketAddr;
use std::str::FromStr;
use std::io::{Write, ErrorKind};
use std::thread;
use std::collections::{BTreeMap};
use self::threadpool::ThreadPool;

// Main struct to handle TCP networking
pub struct TcpNetwork {
    // pending tcp connections, which just accepted but not sent API version and Prime Value
    pending_connections: Slab<TcpReaderConn>,
    pending_write_queue: BTreeMap<Token, Vec<u8>>,

    // server socket for TCP listener
    server_socket: TcpListener,

    // list of channels for TCP reader/writer
    pub reader_channels: Vec<Sender<TcpReaderCommand>>,
    pub writer_channels: Vec<Sender<TcpWriterCommand>>,
    rw_index: usize,

    // keeping current Node [4bytes API version][4 bytes len]token[8 bytes value] combination
    // for direct responses on client/server connections
    // this will be generated inside "init" function
    node_handshake: Vec<u8>
}

impl TcpNetwork {
    pub fn new(server_address: &str, concurrency: usize, net_chan: Sender<NetworkCommand>
            , handshake_info: Vec<u8>, thread_pool: ThreadPool) -> TcpNetwork {
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

        // Starting reader and writer services based on concurrency
        let mut readers: Vec<Sender<TcpReaderCommand>> = vec![];
        let mut writers: Vec<Sender<TcpWriterCommand>> = vec![];
        for _ in 0..concurrency {
            let mut r = TcpReader::new(net_chan.clone());
            readers.push(r.channel());
            let pool = thread_pool.clone();
            thread::spawn(move || {
                r.start(pool);
            });

            let mut w = TcpWriter::new(net_chan.clone());
            let pool = thread_pool.clone();
            writers.push(w.channel());
            thread::spawn(move || {
                w.start(pool);
            });
        }

        TcpNetwork {
            pending_connections: Slab::with_capacity(CONNECTION_COUNT_PRE_ALLOC),
            server_socket: server_socket,
            reader_channels: readers,
            writer_channels: writers,
            rw_index: 0,
            pending_write_queue: BTreeMap::new(),
            node_handshake: handshake_info
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
        entry.insert(TcpReaderConn::new(socket, token, from_server));
    }

    /// Main function for accepting TCP connections in Networking loop
    #[inline(always)]
    fn acceptable(&mut self, poll: &mut Poll) {
        loop {  
            let sock = match self.server_socket.accept() {
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
            self.insert_conn(sock, true, poll);
        };
    }

    /// Main function for reading data from connections, if there is something to read
    #[inline(always)]
    fn readable(&mut self, token: Token, poll: &mut Poll, _: &mut ConnectionsMap) {
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

        // if we got token and value 
        // moving connection from pending to main connections list
        if !token_value.is_none() {
            let (token_str, value) = token_value.unwrap();
            let ref mut conn = self.pending_connections[token];
            conn.conn_token = token_str.clone();
            conn.conn_value = value;
            // adding handshake information to send
            self.pending_write_queue.insert(token, self.node_handshake.clone());
            // making connection writable to send data
            match poll.reregister(&conn.socket, token, Ready::writable(), PollOpt::edge()) {
                Ok(_) => {},
                Err(e) => {
                    Log::error("Unable to make Tcp connection writable from TcpNetworking", e.description());
                    close_conn = true
                }
            }
        }

        if close_conn {
            self.close_connection(token);
        }
    }

    /// Main function for writing to pending connections, while they are in main networking loop
    #[inline(always)]
    fn writable(&mut self, token: Token, poll: &mut Poll, conns: &mut ConnectionsMap) {
        // if we don't have nothing to write for this connection just returning
        if !self.pending_write_queue.contains_key(&token) {
            return;
        }

        let (token_str, value) = {
            let ref mut conn = self.pending_connections[token];
            let mut write_data = self.pending_write_queue.remove(&token).unwrap();

            let write_len = match conn.socket.write(&mut write_data) {
                Ok(n) => n,
                Err(_) => return
            };

            if write_len < write_data.len() {
                self.pending_write_queue.insert(token, Vec::from(&write_data[write_len..]));
                // keeping connection writable because we still have some data to write
                return;
            }

            (conn.conn_token.clone(), conn.conn_value)
        };

        // if we have written handshake information
        // accepting connection and transferring it to reader/writer
        self.accept_transfer_conn(poll, token, token_str.clone(), value, conns);
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

    fn accept_transfer_conn(&mut self, poll: &mut Poll, token: Token, token_str: String, value: u64, conns: &mut ConnectionsMap) -> bool {
        let writer_conn = self.pending_connections[token].make_writer();
        // if we can't create writer connection
        // just closing accepted connection
        if writer_conn.is_none() {
            true
        } else {
            let (index, reader, writer) = self.get_read_writer();
            let mut reader_conn = self.pending_connections.remove(token).unwrap();
            // removing connection from current poll service
            match poll.deregister(&reader_conn.socket) {
                Ok(_) => {},
                Err(e) => Log::error("Unable to deregister Tcp Connection from Networking poll service", e.description())
            }

            let mut writer_conn = writer_conn.unwrap();
            reader_conn.conn_token = token_str.clone();
            writer_conn.conn_token = token_str.clone();

            // Adding connection identity to connection
            // or adding new connection with it
            let conn_identity = ConnectionIdentity{
                writer_index: index,
                socket_type: SocketType::TCP
            };

            if conns.contains_key(&token_str) {
                conns.get_mut(&token_str).unwrap().set_identity(conn_identity);
            } else {
                let mut main_conn = Connection::new(
                    token_str.clone(), value, reader_conn.from_server
                );

                main_conn.set_identity(conn_identity);
                conns.insert(token_str.clone(), main_conn);
            }

            // sending connection to TCP reader
            let mut reader_cmd = TcpReaderCommand::default();
            reader_cmd.cmd = TcpReaderCMD::HandleConnection;
            reader_cmd.conn.push(reader_conn);
            let _ = reader.send(reader_cmd);

            // Sending connection to tcp writer
            let mut writer_cmd = TcpWriterCommand::default();
            writer_cmd.cmd = TcpWriterCMD::HandleConnection;
            writer_cmd.conn.push(writer_conn);
            let _ = writer.send(writer_cmd);

            false
        }
    }
}
