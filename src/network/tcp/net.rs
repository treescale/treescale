#![allow(dead_code)]
extern crate mio;

use self::mio::{Token, Poll, Ready, PollOpt, Events};
use self::mio::channel::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use self::mio::tcp::TcpListener;
use network::tcp::connection::{Connection, ConnReader, ReaderConnection};
use network::tcp::reader::{Reader};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;

const SERVER_TOKEN: Token = Token(1);
const CHANNEL_TOKEN: Token = Token(2);
const TOKEN_START_INDEX: usize = 3;

pub struct Network {
    // Address of server to listen
    server_address: String,

    server_socket: TcpListener,

    // Tcp connections with Key (Reader Token) for using it from Networking loop
    connections: Arc<Mutex<HashMap<Token, Connection>>>,

    // Connection reader for TCP accepted connections but still not authenticated ones
    // when connection is sending authentication token and passing auth process
    // it will be deleted from this map and will be created "Connection" object for full accepted one
    pending_connection_readers: HashMap<Token, ConnReader>,

    // List of readers
    readers: Vec<Sender<Box<Fn(&mut Reader)>>>,
    reader_index: usize,
    sender_channel: Sender<Box<Fn(&mut Network)>>,
}

impl Network {
    pub fn channel(&self) -> Sender<Box<Fn(&mut Network)>> {
        self.sender_channel.clone()
    }

    // transfer connection to one of the reades based on Round Rubin principle
    pub fn transfer_connection(&mut self, token: Token) {
        let conn_reader = match self.pending_connection_readers.remove(&token) {
            Some(c) => c,
            None => return
        };

        if self.reader_index >= self.readers.len() {
            self.reader_index = 0;
        }

        let _ = self.readers[self.reader_index].send(Box::new(move |reader: &mut Reader| {
            reader.reader_connections.insert(token,
                ReaderConnection {

                    socket_reader: ConnReader {
                        socket: match conn_reader.socket.try_clone() {
                            Ok(s) => s,
                            Err(_) => return
                        },

                        read_chunks: Vec::new(),
                        read_length: 0,
                        read_index: 0,
                        socket_token: conn_reader.socket_token
                    },

                    write_queue: Vec::new()
                });
        }));

        self.reader_index += 1;
    }

    pub fn run(&mut self) {
        let poll: Poll = match Poll::new() {
            Ok(p) => p,
            Err(e) => {
                println!("Unable to create Event Loop for Networking -> {:}", e);
                return;
            }
        };

        let addr = match SocketAddr::from_str(self.server_address.as_str()) {
            Ok(r) => r,
            Err(e) => {
                print!("Unable to convert given server address to bindable source {:?}", e);
                return;
            }
        };

        self.server_socket = match TcpListener::bind(&addr) {
            Ok(s) => s,
            Err(e) => {
                print!("Unable to bind server socket {:?}", e);
                return;
            }
        };

        let (sender, reader): (Sender<Box<Fn(&mut Network)>>, Receiver<Box<Fn(&mut Network)>>) = channel();
        self.sender_channel = sender;

        match poll.register(&reader, CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                print!("Unable to Register channel to POLL service {:?}", e);
                return;
            }
        }

        match poll.register(&self.server_socket, SERVER_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                print!("Unable to Register server to POLL service {:?}", e);
                return;
            }
        }

        let mut events: Events = Events::with_capacity(1000);

        loop {
            match poll.poll(&mut events, None) {
                Ok(event_count) => {
                    if event_count == 0 {
                        continue;
                    }

                    for event in events.iter() {
                        let event_token = event.token();
                        if event_token == CHANNEL_TOKEN {
                            match reader.try_recv() {
                                Ok(callback) => {
                                    // callback for command implementation
                                    callback(self);
                                }
                                Err(_) => {}
                            }

                            continue;
                        }

                        let event_kind = event.kind();

                        if event_kind.is_readable() {
                            if event_token == SERVER_TOKEN {
                                self.acceptable(&poll, event_token);
                            }
                            else {
                                self.readable(&poll, event_token);
                            }
                        } else if event_kind.is_writable() {
                            self.writable(&poll, event_token);
                        } else if event_kind.is_hup() || event_kind.is_error() {
                            self.reset_connection(&poll, event_token);
                        }
                    }
                }
                Err(e) => {
                    println!("Error listenning poll event {:?}", e);
                    return;
                }
            }
        }
    }

    fn acceptable(&mut self, poll: &Poll, event_token: Token) {

    }

    fn readable(&mut self, poll: &Poll, event_token: Token) {

    }

    fn writable(&mut self, poll: &Poll, event_token: Token) {

    }

    fn reset_connection(&mut self, poll: &Poll, event_token: Token) {

    }
}
