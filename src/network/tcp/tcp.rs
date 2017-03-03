#![allow(dead_code)]
extern crate slab;
extern crate mio;

use self::mio::Token;
use self::mio::tcp::TcpListener;

type Slab<T> = slab::Slab<T, Token>;

// Main struct to handle TCP networking
pub struct TcpNetwork {
    server_socket: TcpListener,
}
