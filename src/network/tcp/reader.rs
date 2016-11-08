#![allow(dead_code)]
extern crate mio;
extern crate num;

use self::mio::{Token, Poll, Ready, PollOpt, Events};
use self::mio::channel::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use self::num::bigint::BigInt;
use network::tcp::connection::{ReaderConnection, Connection};
use std::net::Shutdown;

const CHANNEL_TOKEN: Token = Token(1);

enum ReaderCommandsTypes {
    StopEventLoop,
}

pub struct ReaderCommand {
    cmd: ReaderCommandsTypes,
}

pub struct Reader {
    // Map of all available connections with
    // values and write Queue for sending data
    pub reader_connections: HashMap<Token, ReaderConnection>,

    sender_channel: Sender<Box<Fn(&mut Reader)>>,
    receiver_channel: Receiver<Box<Fn(&mut Reader)>>,

    // Base allocations for more efficient memory usage
    data_len_buf: Vec<u8>,
    data_chunk: Vec<u8>,

    // General connections map, with mutex for safe thread access
    // in reader we will need this for closing connection or calculating path
    connections: Arc<Mutex<HashMap<Token, Connection>>>,
}

impl Reader {
    pub fn run(&mut self) {
        let poll: Poll = match Poll::new() {
            Ok(p) => p,
            Err(e) => {
                println!("Unable to create Event Loop for Reader -> {:}", e);
                return;
            }
        };

        let (sender, reader): (Sender<Box<Fn(&mut Reader)>>, Receiver<Box<Fn(&mut Reader)>>) = channel();
        self.sender_channel = sender;
        match poll.register(&reader, CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(()) => {}
            Err(_) => {
                // TODO: Hanlde error for channel registration
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

                            // if we don't have connection token, just moving forward
                            if !self.reader_connections.contains_key(&event_token) {
                                self.readable(&poll, event_token);
                            }

                        } else if event_kind.is_writable() {

                            if !self.reader_connections.contains_key(&event_token) {
                                self.writable(&poll, event_token);
                            }

                        } else if event_kind.is_hup() {
                            // TODO: handle HUP type
                        } else if event_kind.is_error() {
                            // TODO: handle ERROR for socket
                        }
                    }
                }
                Err(_) => {
                    // TODO: Handle error here
                    return;
                }
            }
        }
    }

    fn channel(&self) -> Sender<Box<Fn(&mut Reader)>> {
        return self.sender_channel.clone();
    }

    fn readable(&mut self, poll: &Poll, event_token: Token) {
        let mut c = match self.reader_connections.remove(&event_token) {
            Some(f) => f,
            None => return
        };

        let (close_connection, is_final_data, data_buf) = match c.socket_reader.read_data(&mut self.data_len_buf, &mut self.data_chunk) {
            Ok((i, d)) => (false, i, d),
            Err(_) => {
                (true, false, Vec::new())
            }
        };

        if close_connection {
            let _ = c.socket_reader.socket.shutdown(Shutdown::Both);
            // Removing from connections after disconnect
            {
                let mut connections = match self.connections.lock() {
                    Ok(cc) => cc,
                    Err(_) => return
                };

                if connections.contains_key(&event_token) {
                    connections.remove(&event_token);
                }

                // TODO: trigger event about connection close
            }
            return;
        }

        // if we got here then we don't have error in read process
        // so adding back connection to hashmap
        self.reader_connections.insert(event_token, c);

        // if we don't have final data yet, just moving forward
        if !is_final_data {
            return;
        }

        // if we got data handling it
        if data_buf.len() > 0 {
            // TODO: handle data here!!!
        }
    }

    fn writable(&mut self, poll: &Poll, event_token: Token) {
        let mut c = match self.reader_connections.remove(&event_token) {
            Some(f) => f,
            None => return
        };

        let done = match c.flush_data() {
            Ok(done) => done,
            Err(e) => {
                return;
            }
        };

        // if we still have data to write registering as a writable again
        if !done {
            let _ = poll.reregister(&c.socket_reader.socket, event_token, Ready::writable(), PollOpt::edge() | PollOpt::oneshot());
        }

        // if we got here then we don't have error in read process
        // so adding back connection to hashmap
        self.reader_connections.insert(event_token, c);
    }
}
