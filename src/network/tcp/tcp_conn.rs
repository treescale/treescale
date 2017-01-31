#![allow(dead_code)]
extern crate mio;

use self::mio::tcp::TcpStream;
use self::mio::{Token, Poll, Ready, PollOpt};
use std::io::ErrorKind;
use std::io::Read;
use std::mem;

/// Structure for handling TCP connection functionality
pub struct TcpReaderConn {
    // api version for this connection
    pub api_version: u32,
    // TCP stream socket handle
    pub socket: TcpStream,
    // token for event loop
    pub socket_token: Token,
    // unique prime value
    pub value: u64,

    // values for keeping pending data information
    pending_length: usize,
    pending_index: usize,
    pending_data: Vec<Vec<u8>>,
    // for getting number out of bytes we need to read
    // 4 bytes as a u32 (unsigned int32)
    // so we need to keep indexes for that 4 bytes
    endian_bytes: Vec<u8>,
    endian_bytes_index: usize
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
    write_queue_element_index: usize,
}


impl TcpReaderConn {
    pub fn new(socket: TcpStream, token: Token) -> TcpReaderConn {
        TcpReaderConn {
            api_version: 0,
            socket: socket,
            socket_token: token,
            value: 0,
            pending_length: 0,
            pending_index: 0,
            pending_data: vec![],
            endian_bytes: vec![0; 4],
            endian_bytes_index: 0
        }
    }

    #[inline(always)]
    fn read_big_endian(&mut self) -> (bool, Option<u32>) {
        let read_len = match self.socket.read(&mut self.endian_bytes[self.endian_bytes_index..]) {
            Ok(n) => n,
            Err(e) => {
                // if we got WouldBlock, then this is Non Blocking socket
                // and data still not available for this, so it's not a connection error
                if e.kind() == ErrorKind::WouldBlock {
                    return (false, Some(0));
                }

                return (true, None);
            }
        };

        self.endian_bytes_index += read_len;
        // if we still have some data to read
        if self.endian_bytes_index < self.endian_bytes_index {
            return (false, Some(0));
        }

        // if we got here then just setting BigEndian bytes and returning parsed number
        (true, Some(unsafe {
            let a = [self.endian_bytes[0]
                      , self.endian_bytes[1]
                      , self.endian_bytes[2]
                      , self.endian_bytes[3]];
            let endian_num = mem::transmute::<[u8; 4], u32>(a);
            u32::from_be(endian_num)
        }))
    }

    /// Registering TCP connection to given POLL event loop
    #[inline(always)]
    pub fn register(&self, poll: &Poll) -> bool {
        match poll.register(&self.socket, self.socket_token, Ready::readable(), PollOpt::edge()) {
            Ok(_) => true,
            Err(e) => {
                warn!("Unable to register connection to Poll service ! -> {}", e);
                false
            }
        }
    }

    /// Reading API version from TCP socket
    /// Function will return 'None' if there is a
    /// problem with connection and it need to be closed
    #[inline(always)]
    pub fn read_api_version(&mut self) -> Option<bool> {
        let (done, version) = self.read_big_endian();
        if !done {
            return Some(false);
        }

        if version.is_none() {
            return None;
        }

        self.api_version = version.unwrap();

        Some(true)
    }
}
