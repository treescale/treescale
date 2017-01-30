#![allow(dead_code)]
extern crate mio;
extern crate slab;

use self::mio::channel::{channel, Sender, Receiver};
use self::mio::{Poll, Ready, PollOpt, Token, Events};
use self::mio::tcp::{TcpListener};
use network::{NetworkCommand};
use network::tcp::{TcpReaderCommand, TcpReaderCMD, TcpReader, TcpWriterCommand, TcpWriter, TcpReaderConn};
use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;
use std::process;

type Slab<T> = slab::Slab<T, Token>;

const TCP_SERVER_TOKEN: Token = Token(0);
const RECEIVER_CHANNEL_TOKEN: Token = Token(1);

/// Structure for handling TCP networking functionality
pub struct TcpNetwork {
    // channel to base networking for transfering commands
    network_channel: Sender<NetworkCommand>,

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
    pending_connections: Slab<TcpReaderConn>
}

/// Enumeration for commands available for TcpNetworking
pub enum TcpNetworkCMD {

}

/// Base structure for transferring command over loops to TcpNetworking
pub struct TcpNetworkCommand {

}


impl TcpNetwork {
    pub fn new(net_chan: Sender<NetworkCommand>) -> TcpNetwork {
        let (s, r) = channel::<TcpNetworkCommand>();
        TcpNetwork {
            network_channel: net_chan,
            sender_channel: s,
            receiver_channel: r,
            reader_channels: vec![],
            writer_channels: vec![],
            poll: Poll::new().expect("Unable to create TCP network POLL service"),
            readers_index: 0,
            pending_connections: Slab::with_capacity(1024)
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
            let mut r = TcpReader::new(self.sender_channel.clone(), self.network_channel.clone());
            self.reader_channels.push(r.channel());
            thread::spawn(move || {
                r.start();
            });

            let mut w = TcpWriter::new(self.sender_channel.clone(), self.network_channel.clone());
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
                } else if kind.is_writable() {
                    self.writable(token);
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
    fn get_reader(&mut self) -> Sender<TcpReaderCommand> {
        if self.readers_index >= self.reader_channels.len() {
            self.readers_index = 0;
        }

        self.readers_index += 1;
        self.reader_channels[self.readers_index - 1].clone()
    }

    #[inline(always)]
    fn notify(&mut self, command: &mut TcpNetworkCommand) {

    }

    #[inline(always)]
    fn acceptable(&mut self, server_socket: &TcpListener) {
        loop {
            let sock = match server_socket.accept() {
                Ok((s, _)) => s,
                Err(_) => break
            };

            if self.pending_connections.vacant_entry().is_none() {
                let conns_len = self.pending_connections.len();
                self.pending_connections.reserve_exact(conns_len);
            }

            let entry = self.pending_connections.vacant_entry().unwrap();
            let conn = TcpReaderConn::new(sock, entry.index());
            // if we are unable to register connection to this poll service
            // then just moving to the next connection, by just closing this one
            if !conn.register(&self.poll) {
                continue;
            }

            entry.insert(conn);
        }
    }

    #[inline(always)]
    fn readable(&mut self, token: Token) {
        
    }

    #[inline(always)]
    fn writable(&mut self, token: Token) {

    }

    #[inline(always)]
    fn close_connection(&mut self, token: Token) {

    }
}
