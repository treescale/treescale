#![allow(dead_code)]
extern crate mio;

use std::sync::Arc;
use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::Shutdown;
use std::error::Error;

use helper::{Log, NetHelper};

use self::mio::{Token, Poll, PollOpt, Ready};
use self::mio::tcp::TcpStream;

/// Base TCP connection structure
pub struct TcpConnection {
    // current API version for this communication channel
    pub api_version: u32,

    // Socket for handling connection
    pub socket: TcpStream,
    pub socket_token: Token,

    // this connection coming from server or client connection
    pub from_server: bool,

    // token for connection as an identification
    pub conn_token: String,
    pub conn_value: u64,

    // pending data information
    pending_data_len: usize,
    pending_data_index: usize,
    pending_data: Vec<Vec<u8>>,

    // we will be reading also 4 bytes BigEndian number
    // so for not mixing things keeping 4 bytes array in case it will be partial
    pending_endian: Vec<u8>,
    pending_endian_index: usize,

    // this ones would be used for writing data to connection
    // and probably would be used from writer loop
    // data queue for writing it to connection
    writable: VecDeque<Arc<Vec<u8>>>,
    // index for current partial data to write
    writable_data_index: usize,
}

impl TcpConnection {
    /// Making new TCP connection from accepted socket
    #[inline(always)]
    pub fn new(socket: TcpStream, token: Token, from_server: bool) -> TcpConnection {
        TcpConnection {
            api_version: 0,
            socket: socket,
            socket_token: token,
            from_server: from_server,
            conn_token: String::default(),
            conn_value: 0,
            pending_data_len: 0,
            pending_data_index: 0,
            pending_data: vec![],
            pending_endian: vec![0; 4],
            pending_endian_index: 0,
            writable: VecDeque::new(),
            writable_data_index: 0
        }
    }

    #[inline(always)]
    pub fn add_writable_data(&mut self, data: Arc<Vec<u8>>) {
        self.writable.push_back(data);
    }

    /// Registering connection to give POLL service
    #[inline(always)]
    pub fn register(&self, poll: &Poll) -> bool {
        match poll.register(&self.socket, self.socket_token, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {}
            Err(e) => {
                Log::error("Unable to register tcp connection to given poll service", e.description());
                return false;
            }
        }

        true
    }

    /// Making connection writable for given POLL service
    #[inline(always)]
    pub fn make_readable(&self, poll: &Poll) -> bool {
        match poll.reregister(&self.socket, self.socket_token, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {}
            Err(e) => {
                Log::error("Unable to make tcp connection readable for given poll service", e.description());
                return false;
            }
        }

        true
    }

    /// Making connection writable for given POLL service
    #[inline(always)]
    pub fn make_writable(&self, poll: &Poll) -> bool {
        match poll.reregister(&self.socket, self.socket_token, Ready::writable(), PollOpt::edge()) {
            Ok(_) => {}
            Err(e) => {
                Log::error("Unable to make tcp connection writable for given poll service", e.description());
                return false;
            }
        }
        true
    }

    /// Reading Endian number using Networking API
    #[inline(always)]
    pub fn read_endian(&mut self) -> Option<(bool, u32)> {
        let read_len = match self.socket.read(&mut self.pending_endian[self.pending_endian_index..]) {
            Ok(n) => if n == 0 { return None } else { n },
            Err(e) => {
                // if we got WouldBlock, then this is Non Blocking socket
                // and data still not available for this, so it's not a connection error
                if e.kind() == ErrorKind::WouldBlock {
                    return Some((false, 0))
                }

                return None;
            }
        };

        // if we got data less than we expected
        if read_len + self.pending_endian_index < 4 {
            self.pending_endian_index += read_len;
            return Some((false, 0));
        }

        let (parsed, number) = NetHelper::bytes_to_u32(&self.pending_endian, 0);
        // if we are unable to parse given BigEndian
        // then something wrong with connection or API, we should close it
        if !parsed {
            return None;
        }

        // resting index for next time read
        self.pending_endian_index = 0;

        Some((true, number))
    }

    /// Reading API version as a big endian as a first handshake between connections
    /// Will return (False, N) if there is not enough data to parse
    /// Will return None if there is some problem with connection and we need to close it
    #[inline(always)]
    pub fn read_api_version(&mut self) -> Option<(bool, u32)> {
        // API version is actually a BigEndian u32
        // so just reading as a big endian number
        self.read_endian()
    }

    /// Reading connection Token and Prime Value combination as a second phase of handshake
    /// Will return (false, Token, N) if there is not enough data to parse
    /// Will return None if there is connection error and we need to close it
    #[inline(always)]
    pub fn read_token_value(&mut self) -> Option<(bool, String, u64)> {
        // reading BigEndian length of token
        let (done, data) = match self.read_data_once() {
            Some((d, b)) => (d, b),
            None => return None
        };

        // if we still don't have enough data, returning and waiting to a new cycle
        if !done {
            return Some((false, String::default(), 0))
        }

        // our data contains Token and Value
        // where Value is last 8 bytes
        // so len() - 8 should be text length
        if data.len() <= 8 {
            // if we got wrong API closing connection
            return None;
        }

        let text_len = data.len() - 8;

        // Converting our token to string
        let token =  match String::from_utf8(Vec::from(&data[..text_len])) {
            Ok(t) => t,
            Err(e) => {
                Log::error("Unable to convert received Token bytes to string", e.description());
                return None;
            }
        };

        // converting Value bytes to u64
        // if not converted just closing connection, because it is wrong or corrupted API data
        let (converted, value) = NetHelper::bytes_to_u64(&data, text_len);
        if !converted {
            return None;
        }

        Some((true, token, value))
    }

    /// Reading only one part of data which means that only one
    /// Byte chunk would be returned
    /// This is the base function to read data from socket
    #[inline(always)]
    pub fn read_data_once(&mut self) -> Option<(bool, Vec<u8>)> {
        // fist of all getting BigEndian number to determine how many bytes we need to read
        if self.pending_data_len == 0 {
            let (done_endian, data_len) = match self.read_endian() {
                Some(d) => d,
                None => return None
            };

            // returning if we need more data
            if !done_endian {
                return Some((false, vec![]));
            }

            // making data with specific length
            self.pending_data_len = data_len as usize;
            self.pending_data.push(vec![0; self.pending_data_len]);
        }

        // if we got here then we have defined pending_data and total length
        // so we need to read data until pending_data_index is equal to length
        let read_len = match self.socket.read(&mut self.pending_data[0][self.pending_data_index..]) {
            Ok(n) => if n == 0 { return None } else { n },
            Err(e) => {
                // if we got WouldBlock, then this is Non Blocking socket
                // and data still not available for this, so it's not a connection error
                if e.kind() == ErrorKind::WouldBlock {
                    return Some((false, vec![]))
                }

                return None;
            }
        };

        if self.pending_data_index + read_len < self.pending_data_len {
            self.pending_data_index += read_len;
            return Some((false, vec![]))
        }

        // resetting values
        self.pending_data_index = 0;
        self.pending_data_len = 0;

        Some((true, self.pending_data.remove(0)))
    }

    /// Reading all data available in socket
    /// so this will return only if read_once function will send (false, vec![])
    /// This will help to get all data once and then consume it using single event
    #[inline(always)]
    pub fn read_data(&mut self) -> Option<Vec<Vec<u8>>> {
        let mut total: Vec<Vec<u8>> = vec![];
        loop {
            let (done, data) = match self.read_data_once() {
                Some(d) => d,
                None => return None
            };

            // if we need more data then just breaking the loop
            // and returning what we have right now
            if !done {
                break
            }

            // adding data to our pool
            total.push(data);
        }

        Some(total)
    }

    /// Shutting down connection, this would be called before closing connection
    #[inline(always)]
    pub fn close(&self) {
        match self.socket.shutdown(Shutdown::Both) {
            Ok(_) => {},
            Err(e) => Log::error("Error while trying to close connection", e.description())
        }
    }

    /// Main function to write to TCP connection
    /// It will add data to "writable" as a write queue
    #[inline(always)]
    pub fn write(&mut self, data: Arc<Vec<u8>>, poll: &Poll) {
        self.writable.push_back(data);
        self.make_writable(poll);
    }

    /// Tying to flush all data what we have right now in our socket
    /// Returns None if there is a connection error
    /// Returns Some(true) if queue is now empty
    /// Returns Some(false) if we still have something in queue
    pub fn flush(&mut self) -> Option<bool> {
        loop {
            {
                let data = match self.writable.front() {
                    Some(d) => d,
                    None => break // there is no data in queue
                };

                let write_len = match self.socket.write(&data[self.writable_data_index..]) {
                    Ok(n) => n,
                    Err(e) => {
                        // if we got WouldBlock, then this is Non Blocking socket
                        // and data still not available for this, so it's not a connection error
                        if e.kind() == ErrorKind::WouldBlock {
                            return Some(false)
                        }

                        return None;
                    }
                };

                // if socket is unable to write all data that we have
                // then moving forward index and waiting until next time
                if write_len + self.writable_data_index < data.len() {
                    self.writable_data_index += write_len;
                    return Some(false);
                }

                // if we got here then our data is written
                // so we need to reset index for next data
                // current data would be deleted automatically after this cycle
                self.writable_data_index = 0;
            }
            // if data written deleting from front
            self.writable.pop_front();
        }

        Some(true)
    }
}