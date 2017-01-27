#![allow(dead_code)]
extern crate mio;

use self::mio::tcp::TcpStream;
use self::mio::Token;

/// Structure for handling TCP connection functionality
pub struct TcpReaderConn {
    // TCP stream socket handle
    socket: TcpStream,
    // token for event loop
    socket_token: Token,
    // unique prime value
    value: u64,

    // values for keeping pending data information
    pending_length: usize,
    pending_index: usize,
    pending_data: Vec<Vec<u8>>
}

pub struct TcpWriterConn {
    // TCP stream socket handle
    socket: TcpStream,
    // token for event loop
    socket_token: Token,
    // unique prime value
    value: u64,

    // values for keeping write queue
    write_queue: Vec<Vec<u8>>,
    write_queue_element_index: usize
}
