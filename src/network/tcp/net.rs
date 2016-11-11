#![allow(dead_code)]
extern crate mio;

use self::mio::{Token, Poll, Ready, PollOpt, Events};
use self::mio::channel::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use self::mio::tcp::TcpListener;
use network::tcp::connection::{Connection, ReaderConnection};
use network::tcp::reader::{Reader};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::os::unix::io::{AsRawFd, RawFd};
use std::net::Shutdown;

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
    pending_connection_readers: HashMap<Token, ReaderConnection>,

    // List of readers
    readers: Vec<Sender<Box<Fn(&mut Reader)>>>,
    reader_index: usize,
    sender_channel: Sender<Box<Fn(&mut Network)>>,


    // Base allocations for more efficient memory usage
    data_len_buf: Vec<u8>,
    data_chunk: Vec<u8>,
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
                    socket: match conn_reader.socket.try_clone() {
                        Ok(s) => s,
                        Err(_) => return
                    },

                    read_chunks: Vec::new(),
                    read_length: 0,
                    read_index: 0,
                    socket_token: conn_reader.socket_token,
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
                            self.reset_pending_connection(&poll, event_token);
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
        loop {
            match self.server_socket.accept() {
                Ok((socket, _ /* addr - don't need at this point*/ )) => {
                    // Generating raw fd for setting token based on that unique number
                    let token = Token(socket.as_raw_fd() as usize);
                    // registering connection with OneShot for using optimal load on event loop
                    match poll.register(&socket, token, Ready::readable(), PollOpt::edge() | PollOpt::oneshot()) {
                        Ok(_) => {},
                        Err(e) => {
                            println!("Unable to regsiter accepted connection with Poll {:?}", e);
                            continue;
                        }
                    }
                    
                    self.pending_connection_readers.insert(
                        token, 
                        ReaderConnection {
                            socket: socket,
                            read_chunks: Vec::new(),
                            read_length: 0,
                            read_index: 0,
                            socket_token: token,
                            write_queue: Vec::new()
                            });
                }
                Err(_) => return
            }
        }
    }

    fn readable(&mut self, poll: &Poll, event_token: Token) {
        if !self.pending_connection_readers.contains_key(&event_token) {
            return;
        }

        let mut c = match self.pending_connection_readers.remove(&event_token) {
            Some(f) => f,
            None => return
        };
        
        let (close_connection, is_final_data, data_buf) = match c.read_data(&mut self.data_len_buf, &mut self.data_chunk) {
            Ok((i, d)) => (false, i, d),
            Err(_) => (true, false, Vec::new())
        };

        if close_connection {
            // closing socket connection
            let _ = c.socket.shutdown(Shutdown::Both);

            // we already got connection from hashmap,
            // so it will be automatically dealocated on return
            return;
        }

        // if we don't have errors, registering connection back, because
        // it have been deleted from event loop using OneShot enabled
        match poll.reregister(&c.socket, event_token, Ready::readable(), PollOpt::edge() | PollOpt::oneshot()) {
            Ok(_) => {},
            Err(e) => {
                println!("Unable to reregister connection after read process {:?}", e);
                return;
            }
        }

        // if we got here then we don't have error in read process
        // so adding back connection to hashmap
        self.pending_connection_readers.insert(event_token, c);

        // if data is not final reading more
        if !is_final_data {
            return;
        }

        if data_buf.len() > 0 {
            // TODO: handle data here
        }
    }

    fn writable(&mut self, poll: &Poll, event_token: Token) {
        if !self.pending_connection_readers.contains_key(&event_token) {
            return;
        }

        let mut c = match self.pending_connection_readers.remove(&event_token) {
            Some(f) => f,
            None => return
        };

        let done = match c.flush_data() {
            Ok(d) => d,
            Err(_) => return
        };

        // if write is not done yet
        // we need to make this connection writable again
        if !done {
            match poll.reregister(&c.socket, event_token, Ready::writable(), PollOpt::edge() | PollOpt::oneshot()) {
                Ok(_) => {},
                Err(e) => {
                    println!("Unable to reregister connection as a writable after write process {:?}", e);
                    return;
                }
            }
        }

        // if we got here then we don't have error in write process
        // so adding back connection to hashmap
        self.pending_connection_readers.insert(event_token, c);
    }

    fn reset_pending_connection(&mut self, poll: &Poll, event_token: Token) {
        if !self.pending_connection_readers.contains_key(&event_token) {
            return;
        }

        let mut c = match self.pending_connection_readers.remove(&event_token) {
            Some(f) => f,
            None => return
        };

        // closing socket connection
        let _ = c.socket.shutdown(Shutdown::Both);
    }

    fn write_data(&mut self, socket_token: Token, data: Vec<u8>) -> Result<()> {
        let mut c_map = match self.connections.lock() {
            Ok(m) => m,
            Err(e) => return Err(e)
        }
    }
}
