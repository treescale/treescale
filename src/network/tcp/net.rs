#![allow(dead_code)]
extern crate mio;
extern crate slab;

use self::mio::channel::{channel, Sender, Receiver};
use self::mio::{Poll, Ready, PollOpt, Token, Events};
use self::mio::tcp::{TcpListener, TcpStream};
use network::{NetworkCommand, Connection, NetworkCMD};
use network::tcp::{TcpReaderCommand, TcpReaderCMD, TcpReader, TcpWriterCommand, TcpWriter, TcpWriterCMD, TcpReaderConn};
use node::NodeCommand;
use helpers::{encode_number, encode_number64};
use std::net::{SocketAddr, TcpStream as TcpStreamO};
use std::str::FromStr;
use std::thread;
use std::process;
use std::io::Write;
use std::u32::MAX as u32MAX;
use std::sync::Arc;

type Slab<T> = slab::Slab<T, Token>;

const TCP_SERVER_TOKEN: Token = Token(u32MAX as usize);
const RECEIVER_CHANNEL_TOKEN: Token = Token((u32MAX - 1) as usize);

/// Structure for handling TCP networking functionality
pub struct TcpNetwork {
    // channel to base networking for transfering commands
    network_channel: Sender<NetworkCommand>,
    node_channel: Sender<NodeCommand>,

    // Sender and Receiver for handling commands for Networking
    sender_channel: Sender<TcpNetworkCommand>,
    receiver_channel: Receiver<TcpNetworkCommand>,

    // commands channels for sending data to Reader loops
    reader_channels: Vec<Sender<TcpReaderCommand>>,

    // commands channels for sending data to Writer loops
    writer_channels: Vec<Sender<TcpWriterCommand>>,

    // keeping POLL service handle
    poll: Poll,

    // index for making Round Rubin over TCP Readers
    readers_index: usize,

    // List of connections which still didn't sent their base information
    // API version and unique Prime value
    pending_connections: Slab<TcpReaderConn>,

    // keeping current node API version for sending during client requests
    current_api_version: u32,

    // keeping current node Value for sending it during client requests
    current_value: u64
}

/// Enumeration for commands available for TcpNetworking
pub enum TcpNetworkCMD {
    ClientConnection
}

/// Base structure for transferring command over loops to TcpNetworking
pub struct TcpNetworkCommand {
    pub cmd: TcpNetworkCMD,
    pub client_address: String
}


impl TcpNetwork {
    pub fn new(net_chan: Sender<NetworkCommand>, node_chan: Sender<NodeCommand>, api_version: u32, current_value: u64) -> TcpNetwork {
        let (s, r) = channel::<TcpNetworkCommand>();
        TcpNetwork {
            network_channel: net_chan,
            sender_channel: s,
            receiver_channel: r,
            reader_channels: vec![],
            writer_channels: vec![],
            poll: Poll::new().expect("Unable to create TCP network POLL service"),
            readers_index: 0,
            pending_connections: Slab::with_capacity(1024),
            node_channel: node_chan,
            current_api_version: api_version,
            current_value: current_value
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<TcpNetworkCommand> {
        self.sender_channel.clone()
    }

    pub fn start(&mut self, concurrency: usize, server_address: &str) {
        // saving channels for later communication
        // and starting reader/writer services as separate threads
        for _ in 0..concurrency {
            let mut r = TcpReader::new(self.sender_channel.clone(), self.network_channel.clone(), self.node_channel.clone());
            self.reader_channels.push(r.channel());
            thread::spawn(move || {
                r.start();
            });

            let mut w = TcpWriter::new(self.sender_channel.clone(), self.network_channel.clone(), self.node_channel.clone());
            self.writer_channels.push(w.channel());
            thread::spawn(move || {
                w.start();
            });
        }

        match self.poll.register(&self.receiver_channel, RECEIVER_CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                warn!("Unable to register channel receiver for Tcp Networking -> {}", e);
                process::exit(1);
            }
        }

        // making TcpListener for making server socket
        let addr = match SocketAddr::from_str(server_address) {
            Ok(a) => a,
            Err(e) => {
                warn!("Unable to parse given server address {} -> {}", server_address, e);
                return;
            }
        };

        // binding TCP server
        let server_socket = match TcpListener::bind(&addr) {
            Ok(s) => s,
            Err(e) => {
                warn!("Unable to bind TCP Server to given address {} -> {}", server_address, e);
                return;
            }
        };

        match self.poll.register(&server_socket, TCP_SERVER_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                warn!("Unable to register TcpServer socket for Tcp Networking -> {}", e);
                process::exit(1);
            }
        }

        // making events for handling 5K events at once
        let mut events: Events = Events::with_capacity(5000);
        loop {
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.iter() {
                let token = event.token();
                if token == RECEIVER_CHANNEL_TOKEN {
                    // trying to get commands while there is available data
                    loop {
                        match self.receiver_channel.try_recv() {
                            Ok(cmd) => {
                                let mut c = cmd;
                                self.notify(&mut c);
                            }
                            // if we got error, then data is unavailable
                            // and breaking receive loop
                            Err(_) => break
                        }
                    }
                    continue;
                }

                let kind = event.kind();

                if kind.is_readable() {
                    if token == TCP_SERVER_TOKEN {
                        self.acceptable(&server_socket);
                    } else {
                        self.readable(token);
                    }
                } else if kind.is_error() || kind.is_hup() {
                    if token == TCP_SERVER_TOKEN {
                        warn!("Got Error for TCP server, exiting Application");
                        process::exit(1);
                    }

                    // if this error on connection, then we need to close it
                    self.close_connection(token);
                }
            }
        }
    }

    #[inline(always)]
    fn get_reader_writer(&mut self) -> (Sender<TcpReaderCommand>, Sender<TcpWriterCommand>) {
        if self.readers_index >= self.reader_channels.len() {
            self.readers_index = 0;
        }

        self.readers_index += 1;
        (self.reader_channels[self.readers_index - 1].clone(),
        self.writer_channels[self.readers_index - 1].clone())
    }

    fn insert_conn(&mut self, sock: TcpStream, from_server: bool) -> Option<Token> {
        if self.pending_connections.vacant_entry().is_none() {
            let conns_len = self.pending_connections.len();
            self.pending_connections.reserve_exact(conns_len);
        }

        let entry = self.pending_connections.vacant_entry().unwrap();
        let mut conn = TcpReaderConn::new(sock);
        conn.socket_token = entry.index();
        conn.from_server = from_server;
        let token = conn.socket_token;
        // if we are unable to register connection to this poll service
        // then just moving to the next connection, by just closing this one
        match self.poll.register(&conn.socket, conn.socket_token, Ready::readable(), PollOpt::edge()) {
            Ok(_) => true,
            Err(e) => {
                warn!("Unable to register connection to TCP Networking Poll service ! -> {}", e);
                return None
            }
        };

        entry.insert(conn);
        Some(token)
    }

    #[inline(always)]
    fn notify(&mut self, command: &mut TcpNetworkCommand) {
        match command.cmd {
            TcpNetworkCMD::ClientConnection => {
                if command.client_address.len() == 0 {
                    return;
                }

                self.connect(command.client_address.as_str());
            }
        }
    }

    #[inline(always)]
    fn acceptable(&mut self, server_socket: &TcpListener) {
        loop {
            let sock = match server_socket.accept() {
                Ok((s, _)) => s,
                Err(_) => break
            };

            self.insert_conn(sock, true);
        }
    }

    #[inline(always)]
    fn readable(&mut self, token: Token) {
        if !self.pending_connections.contains(token) {
            return;
        }

        let read_res = {
            let ref mut conn = self.pending_connections[token];
            if conn.api_version > 0 {
                Some(true)
            } else {
                conn.read_api_version()
            }
        };

        if read_res.is_none() {
            self.close_connection(token);
            return;
        }

        if !read_res.unwrap() {
            return;
        }

        let read_res = {
            let ref mut conn = self.pending_connections[token];
            if conn.value > 0 {
                Some(true)
            } else {
                conn.read_prime_value()
            }
        };

        if read_res.is_none() {
            self.close_connection(token);
            return;
        }

        if !read_res.unwrap() {
            return;
        }

        // if we got here then now we have API version and Prime value from connection
        // so now we can make TcpWriter from it and transfer both to Reader and Writer services
        self.transfer_connection(token);
    }

    #[inline(always)]
    fn close_connection(&mut self, token: Token) {
        if !self.pending_connections.contains(token) {
            return;
        }

        let conn = self.pending_connections.remove(token).unwrap();
        let _ = self.poll.deregister(&conn.socket);
        // clearing connection memory
        // which will actionally close other socket things
        drop(conn);
    }

    #[inline(always)]
    fn transfer_connection(&mut self, token: Token) {
        if !self.pending_connections.contains(token) {
            return;
        }

        let conn = self.pending_connections.remove(token).unwrap();
        match self.poll.deregister(&conn.socket) {
            Ok(_) => {
                let writer_conn_x = conn.make_writer();
                if writer_conn_x.is_none() {
                    return;
                }

                let mut writer_conn = writer_conn_x.unwrap();

                if writer_conn.from_server {
                    let mut total_buf = vec![0u8; 12];
                    encode_number(&mut total_buf[0..4], self.current_api_version);
                    encode_number64(&mut total_buf[4..12], self.current_value);
                    writer_conn.write_queue.push(Arc::new(total_buf));
                }

                let (reader, writer) = self.get_reader_writer();

                // Making base Connection object to notify base Networking about new connection
                let _ = self.network_channel.send(NetworkCommand {
                    cmd: NetworkCMD::HandleNewConnection,
                    connection: vec![Connection::from_tcp(&conn, writer.clone(), true)],
                    event: vec![],
                    client_address: String::new()
                });

                let _ = reader.send(TcpReaderCommand{
                    cmd: TcpReaderCMD::HandleNewConnection,
                    conn: vec![conn]
                });

                let _ = writer.send(TcpWriterCommand {
                    cmd: TcpWriterCMD::HandleNewConnection,
                    conn: vec![writer_conn],
                    token: vec![],
                    data: vec![],
                });
            },
            Err(e) => {
                warn!("Unable to deregister connection from TcpNetwork POLL service -> {}", e);
            }
        }
    }

    #[inline(always)]
    pub fn connect(&mut self, address: &str) {
        // making Socket address for connecting to it
        let addr = match SocketAddr::from_str(address) {
            Ok(a) => a,
            Err(e) => {
                warn!("Unable to parse given client address {} -> {}", address, e);
                return;
            }
        };

        let mut socket = match TcpStreamO::connect(&addr) {
            Ok(s) => s,
            Err(e) => {
                warn!("Error while trying to connect to client address {} -> {}", address, e);
                return;
            }
        };

        // writing 12 bytes directly to socket, in any case it would accept 12 bytes

        let mut total_buf = vec![0u8; 12];
        encode_number(&mut total_buf[0..4], self.current_api_version);
        encode_number64(&mut total_buf[4..12], self.current_value);
        match socket.write(&mut total_buf) {
            Ok(_) => {},
            Err(e) => {
                warn!("Error while trying to write to newly connected client connection [{}] -> {}", address, e);
                return;
            }
        };

        // after sending base information inserting connection for later usage
        self.insert_conn(match TcpStream::from_stream(socket) {
            Ok(s) => s,
            Err(e) => {
                warn!("Error while trying to convert sync client connection to async [{}] -> {}", address, e);
                return;
            }
        }, false);
    }
}
