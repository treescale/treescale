#![allow(dead_code)]
extern crate mio;

use self::mio::Token;
use self::mio::tcp::TcpStream;
use std::collections::VecDeque;
use std::sync::Arc;
use std::error::Error;
use helper::Log;
use std::net::Shutdown;

pub struct TcpReaderConn {
    // connection API version for handling network upgrades
    pub api_version: u32,

    // Tcp socket and reader token
    pub socket: TcpStream,
    socket_token: Token,

    // this connection coming from server or client connection
    pub from_server: bool,

    // token for connection as an identification
    pub conn_token: String,
    pub conn_value: u64,

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
    pub conn_token: String,

    // data queue for writing it to connection
    writable: VecDeque<Arc<Vec<u8>>>,
    // index for current partial data to write
    writable_data_index: usize
}

impl TcpReaderConn {
    #[inline(always)]
    pub fn new(sock: TcpStream, token: Token, from_server: bool) -> TcpReaderConn {
        TcpReaderConn {
            api_version: 0,
            socket: sock,
            socket_token: token,
            conn_token: String::new(),
            conn_value: 0,
            pending_data_len: 0,
            pending_data_index: 0,
            pending_data: VecDeque::new(),
            pending_endian: vec![],
            pending_endian_index: 0,
            from_server: from_server
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
    pub fn read_endian() -> Option<(bool, u32)> {
        unimplemented!()
        // Some((false, 0))
    }

    #[inline(always)]
    pub fn from_server(&self) -> bool {
        self.from_server
    }

    /// Reading API version as a big endian as a first handshake between connections
    /// Will return (False, N) if there is not enough data to parse
    /// Will return None if there is some problem with connection and we need to close it
    #[inline(always)]
    pub fn read_api_version(&mut self) -> Option<(bool, u32)> {
        unimplemented!()
        // Some((false, 0))
    }

    /// Reading connection Token and Prime Value combination as a second phase of handshake
    /// Will return (false, Token, N) if there is not enough data to parse
    /// Will return None if there is connection error and we need to close it
    #[inline(always)]
    pub fn read_token_value(&mut self) -> Option<(bool, String, u64)> {
        unimplemented!()
        // Some((false, String::default(), 0))
    }

    #[inline(always)]
    pub fn read_data(&mut self) -> Option<(bool, Vec<Vec<u8>>)> {
        unimplemented!()
        // Some((false, vec![]))
    }

    #[inline(always)]
    pub fn close(&self) {
        match self.socket.shutdown(Shutdown::Both) {
            Ok(_) => {},
            Err(e) => Log::error("Error while trying to close connection", e.description())
        }
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
