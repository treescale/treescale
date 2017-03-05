#![allow(dead_code)]
extern crate mio;

use self::mio::Token;
use self::mio::tcp::TcpStream;
use std::collections::VecDeque;
use std::sync::Arc;
use std::error::Error;
use logger::Log;

pub struct TcpReaderConn {
    // Tcp socket and reader token
    socket: TcpStream,
    socket_token: Token,

    // token for connection as an identification
    conn_token: String,

    // pending data information
    pending_data_len: usize,
    pending_data_index: usize,
    pending_data: VecDeque<Vec<u8>>,

    // we will be reading also 4 bytes BigEndian number
    // so for not mixing things keeping 4 bytes array in case it will be partial
    pending_endian: Vec<u8>,
    pending_endian_index: usize
}

pub struct TcpWriterConn {
    // Tcp socket and writer token
    socket: TcpStream,
    socket_token: Token,

    // token for connection as an identification
    conn_token: String,

    // data queue for writing it to connection
    writable: VecDeque<Arc<Vec<u8>>>,
    // index for current partial data to write
    writable_data_index: usize
}

impl TcpReaderConn {
    #[inline(always)]
    pub fn new(sock: TcpStream) -> TcpReaderConn {
        TcpReaderConn {
            socket: sock,
            socket_token: Token(0),
            conn_token: String::new(),
            pending_data_len: 0,
            pending_data_index: 0,
            pending_data: VecDeque::new(),
            pending_endian: vec![],
            pending_endian_index: 0
        }
    }

    #[inline(always)]
    pub fn make_writer(&self) -> Option<TcpWriterConn> {
        let sock = match self.socket.try_clone() {
            Ok(s) => s,
            Err(e) => {
                Log::error("Unable to clone Tcp Connection from reader to writer", e.description());
                return None;
            }
        };

        Some(TcpWriterConn::new(sock, self.conn_token.clone()))
    }

    #[inline(always)]
    pub fn read_endian() {
        // TOOD: start reading endian
    }
}

impl TcpWriterConn {
    #[inline(always)]
    pub fn new(sock: TcpStream, conn_token: String) -> TcpWriterConn {
        TcpWriterConn {
            socket: sock,
            socket_token: Token(0),
            conn_token: conn_token,
            writable: VecDeque::new(),
            writable_data_index: 0
        }
    }
}
