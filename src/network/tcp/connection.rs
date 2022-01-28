extern crate mio;
extern crate num_traits;

use self::num_traits::cast;
use constants::CLIENT_API_VERSION_OFFSET;
use helpers::{Log, NetHelper};
use mio::net::TcpStream;
use mio::{Interest, Poll, Token};
use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::Shutdown;
use std::sync::Arc;

pub struct TcpConnection {
    // current API version for this communication channel
    pub api_version: u32,

    // An actual socket for this connection
    pub socket: TcpStream,
    // Low level token for this connection for the Poll
    pub token: Token,

    // this connection coming from server or client connection
    pub from_server: bool,

    // pending data information
    pending_data_len: usize,
    pending_data_index: usize,
    // pending_data is an array because we need fast data move between contexts instead of copy
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
    pub fn new(socket: TcpStream, token: Token) -> TcpConnection {
        TcpConnection {
            socket,
            token,
            from_server: false,
            api_version: 0,
            pending_data_len: 0,
            pending_data_index: 0,
            pending_data: vec![],
            pending_endian: vec![0; 4],
            pending_endian_index: 0,
            writable: VecDeque::new(),
            writable_data_index: 0,
        }
    }

    /// Registering connection to give POLL service
    #[inline(always)]
    pub fn register(&mut self, poll: &Poll) -> bool {
        match poll
            .registry()
            .register(&mut self.socket, self.token, Interest::READABLE)
        {
            Ok(_) => {}
            Err(e) => {
                Log::error(
                    "Unable to register network.tcp connection to given poll service",
                    e.to_string().as_str(),
                );
                return false;
            }
        }

        true
    }

    /// Making connection writable for given POLL service
    #[inline(always)]
    pub fn make_readable(&mut self, poll: &Poll) -> bool {
        match poll
            .registry()
            .reregister(&mut self.socket, self.token, Interest::READABLE)
        {
            Ok(_) => {}
            Err(e) => {
                Log::error(
                    "Unable to make network.tcp connection readable for given poll service",
                    e.to_string().as_str(),
                );
                return false;
            }
        }

        true
    }

    /// Making connection writable for given POLL service
    #[inline(always)]
    pub fn make_writable(&mut self, poll: &Poll) -> bool {
        match poll
            .registry()
            .reregister(&mut self.socket, self.token, Interest::WRITABLE)
        {
            Ok(_) => {}
            Err(e) => {
                Log::error(
                    "Unable to make network.tcp connection writable for given poll service",
                    e.to_string().as_str(),
                );
                return false;
            }
        }
        true
    }

    /// Main function to write to TCP connection
    /// It will add data to "writable" as a write queue
    #[inline(always)]
    pub fn write(&mut self, data: Vec<u8>, poll: &Poll) {
        let mut length_buffer: Vec<u8> = Vec::new();
        length_buffer.resize(4, 0);
        NetHelper::u32_to_bytes(
            cast(data.len()).expect("Write Data Length Overflow"),
            length_buffer.as_mut_slice(),
            0,
        );
        length_buffer.extend(data);
        self.writable.push_back(Arc::new(length_buffer));
        self.make_writable(poll);
    }

    /// Reading Endian number using Networking API
    #[inline(always)]
    pub fn read_endian(&mut self) -> Option<(bool, u32)> {
        let read_len = match self
            .socket
            .read(&mut self.pending_endian[self.pending_endian_index..])
        {
            Ok(n) => {
                if n == 0 {
                    return None;
                } else {
                    n
                }
            }
            Err(e) => {
                // if we got WouldBlock, then this is Non Blocking socket
                // and data still not available for this, so it's not a connection error
                if e.kind() == ErrorKind::WouldBlock {
                    return Some((false, 0));
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
    pub fn read_api_version(&mut self) -> (bool, u32) {
        // API version is actually a BigEndian u32
        // so just reading as a big endian number
        match self.read_endian() {
            Some((ok, version)) => {
                self.from_server = version > CLIENT_API_VERSION_OFFSET;
                self.api_version = if self.from_server {
                    version - CLIENT_API_VERSION_OFFSET
                } else {
                    version
                };
                (ok, self.api_version)
            }
            None => (false, 0),
        }
    }

    /// Sending API version to connected socket
    #[inline(always)]
    pub fn write_api_version(&mut self, version: u32) {
        let version_bytes: &mut [u8] = &mut [0; 4];
        NetHelper::u32_to_bytes(version, version_bytes, 0);
        self.socket
            .write_all(version_bytes)
            .expect("Unable to write API version");
        self.api_version = version;
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
                None => return None,
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
        let read_len = match self
            .socket
            .read(&mut self.pending_data[0][self.pending_data_index..])
        {
            Ok(n) => {
                if n == 0 {
                    return None;
                } else {
                    n
                }
            }
            Err(e) => {
                // if we got WouldBlock, then this is Non Blocking socket
                // and data still not available for this, so it's not a connection error
                if e.kind() == ErrorKind::WouldBlock {
                    return Some((false, vec![]));
                }

                return None;
            }
        };

        if self.pending_data_index + read_len < self.pending_data_len {
            self.pending_data_index += read_len;
            return Some((false, vec![]));
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
    pub fn read_data(&mut self) -> Option<Vec<u8>> {
        let mut total: Vec<u8> = vec![];
        loop {
            let (done, data) = match self.read_data_once() {
                Some(d) => d,
                None => return None,
            };

            // if we need more data then just breaking the loop
            // and returning what we have right now
            if !done {
                break;
            }

            // adding data to our pool
            total.extend(data);
        }

        Some(total)
    }

    /// Tying to flush all data what we have right now in our socket
    /// Returns None if there is a connection error
    /// Returns Some(true) if queue is now empty
    /// Returns Some(false) if we still have something in queue
    pub fn flush_write(&mut self) -> Option<bool> {
        loop {
            {
                let data = match self.writable.front() {
                    Some(d) => d,
                    None => break, // there is no data in queue
                };

                let write_len = match self.socket.write(&data[self.writable_data_index..]) {
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

    /// Shutting down connection, this would be called before closing connection
    #[inline(always)]
    pub fn close(&mut self) {
        match self.socket.shutdown(Shutdown::Both) {
            Ok(_) => {}
            Err(e) => {
                Log::error(
                    "Error while trying to close connection",
                    e.to_string().as_str(),
                );
            }
        }
    }
}
