#![allow(dead_code)]
extern crate slab;
extern crate mio;

use self::mio::Token;

type Slab<T> = slab::Slab<T, Token>;

// Main struct to handle TCP networking
pub struct TcpNetwork {
    
}
