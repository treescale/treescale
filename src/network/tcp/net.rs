#![allow(dead_code)]
extern crate mio;

use self::mio::{Token};
use self::mio::channel::Sender;
use std::sync::{Arc};
use network::tcp::connection::Connection;
use network::tcp::reader::{Reader, MutexQueue};
use std::collections::HashMap;

const SERVER_TOKEN: Token = Token(1);
const TOKEN_START_INDEX: usize = 1;

pub struct Network {
    // Address of server to listen
    server_address: String,

    // token for server event loop
    server_token: Token,

    // First position for connections before accepting them
    pending_connections: HashMap<Token, Connection>,

    // Tcp connections with Key (Reader Token) for using it from Networking loop
    connections: HashMap<Token, Connection>,

    // List of readers
    readers: Vec<Sender<Reader>>,
    reader_write_queue: Vec<Arc<MutexQueue<Token, u8>>>
}
