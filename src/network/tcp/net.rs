#![allow(dead_code)]
extern crate mio;
extern crate num;
extern crate log;

use std::sync::{Arc, RwLock};
use std::collections::BTreeMap;
use self::mio::{Token, Poll, Ready, PollOpt, Events};
use self::mio::tcp::TcpListener;
use self::mio::channel::{channel, Sender, Receiver};
use network::tcp::{TcpConnection, TcpReaderCommand, TcpReader};
use event::{EventHandlerCommand};
use std::thread;
use std::net::SocketAddr;
use std::str::FromStr;
use std::process;
use std::io::{Read, ErrorKind};

const TCP_SERVER_TOKEN: Token = Token(0);
const RECEIVER_CHANNEL_TOKEN: Token = Token(1);

pub enum TcpNetworkCMD {

}

pub struct TcpNetworkCommand {
    cmd: TcpNetworkCMD
}

pub struct TcpNetwork {
    // base connections list
    pub connections: Arc<RwLock<BTreeMap<Token, TcpConnection>>>,
    // connections which are still not accepted
    pending_connections: BTreeMap<Token, TcpConnection>,

    sender_channel: Sender<TcpNetworkCommand>,
    receiver_channel: Receiver<TcpNetworkCommand>,
    // channel for triggering events from networking
    event_handler_channel: Sender<EventHandlerCommand>,
    // vector of channels for sending commands to TcpReaders
    reader_channels: Vec<Sender<TcpReaderCommand>>,
    // basic Round Rubin load balancer index for readers
    reader_channel_index: usize,

    // buffer for reading chunked data
    // making allocation once for performance
    data_buffer: Vec<u8>
}

impl TcpNetwork {
    pub fn new(event_channel: Sender<EventHandlerCommand>) -> TcpNetwork {
        let (s, r) = channel::<TcpNetworkCommand>();
        TcpNetwork {
            connections: Arc::new(RwLock::new(BTreeMap::new())),
            pending_connections: BTreeMap::new(),
            reader_channels: Vec::new(),
            event_handler_channel: event_channel,
            sender_channel: s,
            receiver_channel: r,
            reader_channel_index: 0,

            // allocation buffer with 5K bytes
            // we don't need more for TcpNetwork
            data_buffer: Vec::with_capacity(5000)
        }
    }

    pub fn channel(&self) -> Sender<TcpNetworkCommand> {
        self.sender_channel.clone()
    }

    // base function for running TcpNetwork service with TcpReaders
    pub fn run(&mut self, server_address: &str, readers_count: usize) {
        self.reader_channels.reserve(readers_count);
        for i in 0..readers_count {
            let mut r = TcpReader::new(self.connections.clone());
            self.reader_channels[i] = r.channel();
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

        // making poll and registering base handlers
        let poll = match Poll::new() {
            Ok(p) => p,
            Err(e) => {
                warn!("Error creating Poll service -> {}", e);
                return;
            }
        };

        match poll.register(&server_socket, TCP_SERVER_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                warn!("Unable to register server socket to Poll service -> {}", e);
                return;
            }
        }

        match poll.register(&self.receiver_channel, RECEIVER_CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                warn!("Unable to register receiver channel to Poll service -> {}", e);
                return;
            }
        }

        // making events for handling 5K events at once
        let mut events: Events = Events::with_capacity(5000);

        loop {
            let event_count = poll.poll(&mut events, None).unwrap();
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
                                self.notify(&poll, &mut c);
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
                    // self.close_connection(token, true);
                    continue;
                }

                if kind == Ready::readable() {
                    if token == TCP_SERVER_TOKEN {
                        self.acceptable(&poll, &server_socket);
                    } else {
                        self.readable(&poll, token);
                    }
                    continue;
                }

                if kind == Ready::writable() {
                    self.writable(&poll, token);
                    continue;
                }
            }

        }
    }

    #[inline(always)]
    fn acceptable(&mut self, poll: &Poll, server_sock: &TcpListener) {
        loop {
            match server_sock.accept() {
                Ok((sock, _)) => {
                    let conn = TcpConnection::new(sock);
                    match poll.register(&conn.socket, conn.socket_token, Ready::readable(), PollOpt::edge()) {
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
                // if we got error on server accept process
                // we need to break accept loop and wait until new connections
                // would be available in event loop
                Err(_) => break
            }
        }
    }

    #[inline(always)]
    fn notify(&mut self, poll: &Poll, command: &mut TcpNetworkCommand) {
        match command.cmd {

        }
    }

    #[inline(always)]
    fn readable(&mut self, poll: &Poll, token: Token) {
        let mut delete_conn = false;
        {
            let mut conn =  match self.pending_connections.get_mut(&token) {
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
                            return;
                        }

                        delete_conn = true;
                    }
                }
            }

            // if all ok with reading API version
            if !delete_conn {
                match conn.socket.read(&mut self.data_buffer) {
                    Ok(rsize) => {
                        let (data, leave_open) = conn.handle_data(&self.data_buffer, rsize);
                        // checking if we need to close connection or not
                        if leave_open {

                        } else {
                            delete_conn = true;
                        }
                    },
                    Err(e) => {
                        // if we got WouldBlock, then this is Non Blocking socket
                        // and data still not available for this, so it's not a connection error
                        if e.kind() == ErrorKind::WouldBlock {
                            return;
                        }

                        delete_conn = true;
                    }
                };
            }
        }

        if delete_conn {
            // this will also close connection, because
            // tcp socket would be deallocated
            self.pending_connections.remove(&token);
        }
    }

    #[inline(always)]
    fn writable(&mut self, poll: &Poll, token: Token) {

    }
}
