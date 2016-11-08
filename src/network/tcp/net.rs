#![allow(dead_code)]
extern crate mio;

use self::mio::{Token};
use self::mio::channel::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use network::tcp::connection::{Connection, ConnReader, ReaderConnection};
use network::tcp::reader::{Reader};
use std::collections::HashMap;

const SERVER_TOKEN: Token = Token(1);
const TOKEN_START_INDEX: usize = 2;

pub struct Network {
    // Address of server to listen
    server_address: String,

    // Tcp connections with Key (Reader Token) for using it from Networking loop
    connections: Arc<Mutex<HashMap<Token, Connection>>>,

    // Connection reader for TCP accepted connections but still not authenticated ones
    // when connection is sending authentication token and passing auth process
    // it will be deleted from this map and will be created "Connection" object for full accepted one
    pending_connection_readers: HashMap<Token, ConnReader>,

    // List of readers
    readers: Vec<Sender<Box<Fn(&mut Reader)>>>,
    reader_index: usize,
    sender_channel: Sender<fn(&Network)>,
}

impl Network {
    pub fn channel(&self) -> Sender<fn(&Network)> {
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

    }
}
