extern crate mio;

use self::mio::{Token};
use self::mio::tcp::TcpStream;
use std::sync::Arc;
/// Base networking connection struct
/// this wouldn't contain TcpStream
/// main IO operations would be done in Reader Threads
pub struct TcpConnection {

}

/// This struct mainly for making IO for TCP connections
pub struct TcpReaderConn {
    pub token: Token,

    // connection socket for Read/Write operations
    pub socket: TcpStream,

    // fields for handling partial data read
    read_data_queue: Vec<Vec<u8>>,
    read_data_index: usize,
    read_data_len: usize,

    // Write data queue for partial data write
    // when socket becomming writable
    pub write_queue: Vec<Arc<Vec<u8>>>
}

impl TcpReaderConn {
    pub fn new(sock: TcpStream, token: Token) -> TcpReaderConn {
        TcpReaderConn {
            socket: sock,
            read_data_queue: Vec::new(),
            read_data_index: 0,
            read_data_len: 0,
            write_queue: Vec::new(),
            token: token
        }
    }
}
