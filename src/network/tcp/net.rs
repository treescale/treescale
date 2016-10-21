extern crate mio;

use self::mio::{Token};
use std::sync::{Arc, Mutex};
use network::tcp::connection::Connection;

pub struct Network {
    // Address of server to listen
    server_address: String,

    // token for server event loop
    server_token: Token,

    // Tcp connections for using it from Networking loop
    connections: Arc<Mutex<Vec<Connection>>>
}