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
    // api version for this connection
    pub api_version: u32,
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
        if self.endian_bytes_index > 4 {
            warn!("Pending data index bigger than pending data length, closing connection -> Pending Index: {} , Data Lenght {}"
            , self.endian_bytes_index, 4);
            return (true, None);
        }
        // if we still have some data to read
        if self.endian_bytes_index < 4 {
            return (false, Some(0));
        }

        // if we got here then just setting BigEndian bytes and returning parsed number
        self.endian_bytes_index = 0;
        (true, Some(unsafe {
            let a = [self.endian_bytes[0]
                      , self.endian_bytes[1]
                      , self.endian_bytes[2]
                      , self.endian_bytes[3]];
            let endian_num = mem::transmute::<[u8; 4], u32>(a);
            self.endian_bytes = vec![0; 4];
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

    /// Reading Unique Prime Value from TCP Socket
    /// Function will return 'None' if there is a
    /// problem with connection and it need to be closed
    #[inline(always)]
    pub fn read_prime_value(&mut self) -> Option<bool> {
        // trying to read Prime Value as 8 bytes
        if self.pending_length == 0 {
            self.pending_length = 8;
            self.pending_data = vec![vec![0; 8]];
        }

        let read_len = match self.socket.read(&mut self.pending_data[0][self.pending_index..]) {
            Ok(n) => n,
            Err(e) => {
                // if we got WouldBlock, then this is Non Blocking socket
                // and data still not available for this, so it's not a connection error
                if e.kind() == ErrorKind::WouldBlock {
                    return Some(false);
                }

                return None;
            }
        };

        self.pending_index += read_len;
        if self.pending_index > self.pending_length {
            warn!("Pending data index bigger than pending data length, closing connection -> Pending Index: {} , Data Lenght {}"
            , self.pending_index, self.pending_length);
            return None;
        }

        if self.pending_index < self.pending_length {
            return Some(false);
        }

        // if we got here then we have all data
        self.value = unsafe {
            let data = self.pending_data.remove(0);
            let a: [u8; 8] = [data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]];
            u64::from_be(mem::transmute::<[u8; 8], u64>(a))
        };

        self.pending_length = 0;
        self.pending_index = 0;
        Some(true)
    }

    /// Reading next chunck of data from TCP socket
    /// It is based on our [BigEndian Length]:[Raw Data] API combination
    #[inline(always)]
    pub fn read_data(&mut self) -> (bool, Option<Vec<u8>>) {
        if self.pending_length == 0 {
            let (done, length) = self.read_big_endian();
            if !done {
                return (false, None);
            }

            // if there is a problem reading big endian number
            // notifying to close connection
            if length.is_none() {
                return (true, None);
            }

            // if we got here then we have pending data length
            self.pending_length = length.unwrap() as usize;
            self.pending_data = vec![vec![0; self.pending_length]];
        }

        let read_len = match self.socket.read(&mut self.pending_data[0][self.pending_index..]) {
            Ok(n) => n,
            Err(e) => {
                // if we got WouldBlock, then this is Non Blocking socket
                // and data still not available for this, so it's not a connection error
                if e.kind() == ErrorKind::WouldBlock {
                    return (false, None);
                }

                return (true, None);
            }
        };

        self.pending_index += read_len;
        if self.pending_index > self.pending_length {
            warn!("Pending data index bigger than pending data length, closing connection -> Pending Index: {} , Data Lenght {}"
            , self.pending_index, self.pending_length);
            return (true, None);
        }

        if self.pending_index < self.pending_length {
            return (false, None);
        }

        self.pending_length = 0;
        self.pending_index = 0;
        (true, Some(self.pending_data.remove(0)))
    }

    /// Making writer socket out of existing information from TcpReaderConn
    /// This connection later would be shared between one of the TcpWriters
    #[inline(always)]
    pub fn make_writer(&self) -> Option<TcpWriterConn> {
        if self.api_version == 0 || self.value == 0 {
            return None;
        }

        match self.socket.try_clone() {
            Ok(s) => {
                Some(TcpWriterConn{
                    api_version: self.api_version,
                    value: self.value,
                    socket: s,
                    socket_token: self.socket_token,
                    write_queue: vec![],
                    write_queue_element_index: 0
                })
            },
            Err(e) => {
                warn!("Unable to clone socket for making writer service ! -> {}", e);
                None
            }
        }
    }
}

impl TcpWriterConn {

}
