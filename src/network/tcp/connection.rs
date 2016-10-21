extern crate mio;
extern crate num;

use self::mio::{Token};
use self::mio::tcp::TcpStream;
use self::num::bigint::BigInt;
use std::sync::{Arc, Mutex};

pub struct Connection {
    node_token: String,
    reader_index: usize,
    reader_token: Token,
    value: BigInt,

    // reader connection mutex for accessing write queue
    reader_conn: Arc<Mutex<ReaderConnection>>
}

pub struct ReaderConnection {
    loop_token: Token,
    socket: TcpStream,
    writable_data: Vec<Vec<u8>>,

    // partial read variables
    read_chunks: Vec<Vec<u8>>,
    read_length: usize,
    read_index: usize,
}
