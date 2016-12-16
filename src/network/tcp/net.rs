#![allow(dead_code)]
extern crate mio;
extern crate num;
extern crate log;
extern crate byteorder;

use std::sync::{Arc, RwLock};
use std::collections::BTreeMap;
use self::mio::{Token, Poll, Ready, PollOpt, Events};
use self::mio::tcp::{TcpListener, TcpStream};
use self::mio::channel::{Sender, Receiver, channel};
use network::tcp::{TOKEN_VALUE_SEP, TcpConnValue, TcpConn, TcpReaderCommand, TcpReaderCMD, TcpReader};
use self::num::{BigInt};
use std::str::FromStr;
use std::process;
use event::*;
use std::net::{SocketAddr};
use std::io::{Result, ErrorKind, Error};
use std::thread;
use self::byteorder::{BigEndian, ByteOrder};

const TCP_SERVER_TOKEN: Token = Token(0);
const RECEIVER_CHANNEL_TOKEN: Token = Token(1);
const CURRENT_API_VERSION: u32 = 1;

enum TcpNetworkCMD {
    HandleClientConnection
}

pub struct TcpNetworkCommand {
    cmd: TcpNetworkCMD,
    socket: Vec<TcpStream>
}

pub struct TcpNetwork {
    // Base connections which are available and accepted
    connections: Arc<RwLock<Vec<TcpConnValue>>>,

    // Socket based connections which are accepted from TCP server
    // but not accepted from application
    pending_connections: BTreeMap<Token, TcpConn>,

    // token for current networking/node
    current_token: String,
    current_value: BigInt,
    current_value_square: BigInt,

    sender_channel: Sender<TcpNetworkCommand>,
    receiver_channel: Receiver<TcpNetworkCommand>,
    // channel for triggering events from networking
    event_handler_channel: Sender<EventHandlerCommand>,
    // vector of channels for sending commands to TcpReaders
    reader_channels: Vec<Sender<TcpReaderCommand>>,
    // basic Round Rubin load balancer index for readers
    reader_channel_index: usize,
    // base poll object
    poll: Poll
}

impl TcpNetwork {
    pub fn new(token: String, value: String, event_chan: Sender<EventHandlerCommand>) -> TcpNetwork {
        let v = match BigInt::from_str(value.as_str()) {
            Ok(vv) => vv,
            Err(e) => {
                warn!("Unable to convert current node value to BigInt from networking -> {}", e);
                process::exit(1);
            }
        };

        let (s, r) = channel::<TcpNetworkCommand>();

        TcpNetwork {
            connections: Arc::new(RwLock::new(Vec::new())),
            pending_connections: BTreeMap::new(),
            current_token: token,
            current_value: v.clone(),
            current_value_square: v.clone() * v.clone(),
            sender_channel: s,
            receiver_channel: r,
            event_handler_channel: event_chan,
            reader_channels: Vec::new(),
            reader_channel_index: 0,
            poll: match Poll::new() {
                Ok(p) => p,
                Err(e) => {
                    warn!("Unable to create Poll service from networking -> {}", e);
                    process::exit(1);
                }
            }
        }
    }

    pub fn channel(&self) -> Sender<TcpNetworkCommand> {
        self.sender_channel.clone()
    }

    pub fn run(&mut self, server_address: &str, readers_count: usize) {
        self.reader_channels.reserve(readers_count);
        let mut readers: Vec<TcpReader> = vec![];
        for i in 0..readers_count {
            let mut r = TcpReader::new(self.connections.clone(), self.event_handler_channel.clone());
            r.reader_index = i;
            self.reader_channels[i] = r.channel();
            readers.push(r);
        }

        // setting channels and start reader thread
        loop {
            let mut r = match readers.pop() {
                Some(r) => r,
                None => break
            };

            // setting channels here
            r.reader_channels = self.reader_channels.clone();

            thread::spawn(move || {
                r.run();
            });
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
                warn!("Unable to register server socket to Poll service -> {}", e);
                return;
            }
        }

        match self.poll.register(&self.receiver_channel, RECEIVER_CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                warn!("Unable to register receiver channel to Poll service -> {}", e);
                return;
            }
        }

        // making events for handling 5K events at once
        let mut events: Events = Events::with_capacity(5000);
        loop {
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.into_iter() {
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

                if kind == Ready::error() || kind == Ready::hup() {
                    if token == TCP_SERVER_TOKEN {
                        warn!("Got Error for TCP server, exiting Application");
                        process::exit(1);
                    }
                    // if this error on connection, then we need to close it
                    self.close_connection(token);
                    continue;
                }

                if kind == Ready::readable() {
                    if token == TCP_SERVER_TOKEN {
                        self.acceptable(&server_socket);
                    } else {
                        self.readable(token);
                    }
                    continue;
                }

                if kind == Ready::writable() {
                    self.writable(token);
                    continue;
                }
            }

        }
    }

    #[inline(always)]
    fn notify(&mut self, command: &mut TcpNetworkCommand) {
        match command.cmd {
            TcpNetworkCMD::HandleClientConnection => {
                let socket = match command.socket.pop() {
                    Some(c) => c,
                    None => return
                };

                // adding connection here
                self.add_pending_conn(socket, true);
            }
        }
    }

    #[inline(always)]
    fn acceptable(&mut self, listener: &TcpListener) {
        loop {
            match listener.accept() {
                Ok((sock, _)) => {
                    self.add_pending_conn(sock, false);
                }
                // if we got error on server accept process
                // we need to break accept loop and wait until new connections
                // would be available in event loop
                Err(_) => break
            }
        }
    }

    #[inline(always)]
    fn add_pending_conn(&mut self, socket: TcpStream, from_client: bool) {
        let mut conn = TcpConn::new(socket);
        conn.from_server = !from_client;
        let mut ready_state = Ready::readable();
        if from_client {
            ready_state = ready_state | Ready::writable();
            self.write_handshake_info(&mut conn);
        }

        match self.poll.register(&conn.socket, conn.socket_token, ready_state, PollOpt::edge()) {
            Ok(_) => {
                // inserting connection as a pending
                self.pending_connections.insert(conn.socket_token, conn);
            }

            Err(e) => {
                // after this accepted connection would be automatically deleted
                // by closures deallocation
                warn!("Unable to register accepted connection -> {}", e);
            }
        }
    }

    #[inline(always)]
    fn write_handshake_info(&self, conn: &mut TcpConn) {
        // if we got here then we made successfull connection with server
        // now we need to write our API version
        let mut write_data = [0; 4];
        BigEndian::write_u32(&mut write_data, CURRENT_API_VERSION);
        let mut send_data = Vec::new();
        send_data.extend_from_slice(&write_data);
        conn.add_writable_data(Arc::new(send_data));

        let token_value = (self.current_token.clone() + TOKEN_VALUE_SEP.to_string().as_str() + self.current_value.to_str_radix(10).as_str())
                            .into_bytes();

        conn.add_writable_data(Arc::new(token_value));
    }

    #[inline(always)]
    fn readable(&mut self, token: Token) {
        // when we will return functuin without inserting back
        // this connection would be deallocated and would be automatically closed
        let mut conn =  match self.pending_connections.remove(&token) {
            Some(c) => c,
            None => return
        };

        // if we yet don't have an api version
        // reading it
        if conn.api_version <= 0 {
            match conn.read_api_version() {
                Ok(is_done) => {
                    // if we need more data for getting API version
                    // then wiating until socket would become readable again
                    if !is_done {
                        return;
                    }
                },
                Err(e) => {
                    // if we got WouldBlock, then this is Non Blocking socket
                    // and data still not available for this, so it's not a connection error
                    if e.kind() == ErrorKind::WouldBlock {
                        self.pending_connections.insert(token, conn);
                    }

                    return;
                }
            }
        }

        let (conn_token, conn_value, is_done) = match conn.read_token_value() {
            Ok((t,v,d)) => (t,v,d),
            Err(e) => {
                warn!("Error while reading connection token, closing connection -> {}", e);
                return;
            }
        };

        // if we got token and value
        // setting them up, and sending event to User level
        // for authenticating this connection
        if is_done {
            // deregistering connection from Networking loop, because we don't want to receive data anymore
            // until this connection is not accepted
            let _ = self.poll.deregister(&conn.socket);

            // making connection value
            // which would be transferred to Reader
            conn.add_conn_value(token, conn_token.clone(), conn_value.clone());

            if conn.from_server {
                let _ = self.event_handler_channel.send(EventHandlerCommand {
                    cmd: EventHandlerCMD::TriggerFromEvent,
                    event: Arc::new(Event{
                        name: String::from(EVENT_ON_PENDING_CONNECTION),
                        from: conn_token.clone(),
                        target: String::new(),
                        data: conn_value,
                        path: String::new(),
                        public_data: String::new()
                    })
                });
            }
            else {
                // if this connection is from client, then we don't need to check it using User space code
                // just accepting connection after we have server node information

                self.accept_conn(token);
            }
        }

        // if we got here then all operations done
        // adding back connection for keeping it
        self.pending_connections.insert(token, conn);
    }

    #[inline(always)]
    fn writable(&mut self, token: Token) {
        // when we will return functuin without inserting back
        // this connection would be deallocated and would be automatically closed
        let mut conn =  match self.pending_connections.remove(&token) {
            Some(c) => c,
            None => return
        };

        let is_done = match conn.flush_write_queue() {
            Ok(d) => d,
            Err(e) => {
                warn!("Connection Write error, closing connection -> {}", e);
                return;
            }
        };

        // if we done with writing data
        // reregistering connection only readable again
        if is_done {
            match self.poll.reregister(&conn.socket, token, Ready::readable(), PollOpt::edge()) {
                Ok(_) => {},
                Err(e) => {
                    warn!("Unable to reregister connection as writable, closing connection -> {}", e);
                    return;
                }
            }
        }

        // if we got here then all operations done
        // adding back connection for keeping it
        self.pending_connections.insert(token, conn);
    }

    #[inline(always)]
    pub fn accept_conn(&mut self, token: Token) {
        let mut conn =  match self.pending_connections.remove(&token) {
            Some(c) => c,
            None => return
        };

        if conn.from_server {
            self.write_handshake_info(&mut conn);
        }

        match self.get_reader().send(TcpReaderCommand {
            cmd: TcpReaderCMD::HandleConnection,
            conn_value: match conn.pop_conn_value() {
                Some(c) => vec![c],
                None => vec![]
            },
            conn: vec![conn],
            data: vec![],
            socket_token: vec![],
            tokens: vec![],
            event: vec![]
        }) {
            Ok(_) => {},
            Err(_) => {
                warn!("Error while trying to send Reader Command from Networking for connection accept, so closing connection");
                return;
            }
        };
    }

    #[inline(always)]
    fn get_reader(&mut self) -> Sender<TcpReaderCommand> {
        if self.reader_channel_index >= self.reader_channels.len() {
             self.reader_channel_index = 0;
        }

        let r = self.reader_channels[self.reader_channel_index].clone();
        self.reader_channel_index += 1;
        return r;
    }

    #[inline(always)]
    fn close_connection(&mut self, token: Token) {
        // deleting connection from our map, it would be deleted automatically
        self.pending_connections.remove(&token);
    }

    pub fn connect(&mut self, address: &str) -> Result<()> {
        // making TcpListener for making server socket
        let addr = match SocketAddr::from_str(address) {
            Ok(a) => a,
            Err(_) => return Err(Error::new(ErrorKind::AddrNotAvailable, "Unable to make address lookup"))
        };

        let _ = self.sender_channel.send(TcpNetworkCommand {
            cmd: TcpNetworkCMD::HandleClientConnection,
            socket: vec![match TcpStream::connect(&addr) {
                Ok(s) => s,
                Err(e) => return Err(e)
            }]
        });

        Ok(())
    }

    // emit event to given path from Event object and/or to provided connection tokens
    // if we are using API Clients then they wouldn't have Prime values
    pub fn emit(&mut self, ev: Event, tokens: Vec<String>) -> bool {
        let _ = self.get_reader().send(TcpReaderCommand {
            cmd: TcpReaderCMD::WriteDataWithPath,
            conn: vec![],
            conn_value: vec![],
            data: vec![],
            socket_token: vec![],
            event: vec![ev],
            tokens: tokens
        });

        true
    }
}
