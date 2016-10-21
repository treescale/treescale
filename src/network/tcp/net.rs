extern crate mio;

use self::mio::{Token};

pub struct Network {
    // Address of server to listen
    server_address: String,

    // token for server event loop
    server_token: Token,

    
}