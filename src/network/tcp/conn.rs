#![allow(dead_code)]
extern crate mio;
extern crate byteorder;

use self::mio::{Token};
use self::mio::tcp::TcpStream;
use std::sync::Arc;
use std::io::{Result, Read, ErrorKind, Error};
use std::io::Cursor;
use self::byteorder::{BigEndian, ReadBytesExt};

/// Max length for individual message is 30mb
const MAX_MESSAGE_DATA_LEN: usize = 30000000;

/// Read buffer size 64KB
const READ_BUFFER_SIZE: usize = 65000;

pub struct TcpWritableData {
    pub buf: Arc<Vec<u8>>,
    pub offset: usize
}

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
    read_data_len_buf: Vec<u8>,

    // Write data queue for partial data write
    // when socket becomming writable
    pub write_queue: Vec<TcpWritableData>
}

impl TcpReaderConn {
    pub fn new(sock: TcpStream, token: Token) -> TcpReaderConn {
        TcpReaderConn {
            socket: sock,
            read_data_queue: Vec::new(),
            read_data_index: 0,
            read_data_len: 0,
            read_data_len_buf: Vec::new(),
            write_queue: Vec::new(),
            token: token
        }
    }

    pub fn read_data(&mut self) -> Result<Vec<Arc<Vec<u8>>>> {
        let mut ret_data: Vec<Arc<Vec<u8>>> = Vec::new();

        // reading data until socket have pending buffer to read
        loop {

            // if we are starting to read new data, getting first 4 bytes, for handling data length
            if self.read_data_len == 0 {
                // clearing length buffer for getting new data to it
                if self.read_data_len_buf.len() >= 4 {
                    self.read_data_len_buf.clear();
                }

                let mut len_buf: Vec<u8> = vec![0; 4 - self.read_data_len_buf.len()];
                let read_len = match self.socket.read(&mut len_buf) {
                    Ok(s) => {
                        // We got EOF here
                        if s == 0 {
                            return Err(Error::new(ErrorKind::ConnectionReset, "Connection closed !"));
                        }

                        s
                    },
                    Err(e) => {
                        // if we got WouldBlock, then this is Non Blocking socket
                        // and data still not available for this, so it's not a connection error
                        if e.kind() == ErrorKind::WouldBlock {
                            return Ok(ret_data);
                        }

                        return Err(e);
                    }
                };

                // keeping only read data
                len_buf.truncate(read_len);

                self.read_data_len_buf.append(&mut len_buf);

                // if we don't have 4 bytes for data length
                // trying to read again
                if self.read_data_len_buf.len() != 4 {
                    return Ok(ret_data);
                }

                let mut rdr = Cursor::new(&self.read_data_len_buf);
                self.read_data_len =  match rdr.read_u32::<BigEndian>() {
                    Ok(s) => s as usize,
                    Err(_) => return Ok(ret_data)
                };

                if self.read_data_len > MAX_MESSAGE_DATA_LEN {
                    self.read_data_len = 0;
                    self.read_data_index = 0;
                    return Err(Error::new(ErrorKind::InvalidData, "Wrong Data API: Message length is bigger than MAX. message size!"));
                }
            }

            let mut potential_read_len = self.read_data_len - self.read_data_index;
            // we don't want to overflow memory at once
            // reading data part by part if it is bigger than READ_BUFFER_SIZE
            if potential_read_len > READ_BUFFER_SIZE {
                potential_read_len = READ_BUFFER_SIZE;
            }

            let mut data_chunk: Vec<u8> = vec![0; potential_read_len];

            let read_len = match self.socket.read(&mut data_chunk) {
                Ok(size) => {
                    // We got EOF here
                    if size == 0 {
                        return Err(Error::new(ErrorKind::ConnectionReset, "Connection closed !"));
                    }

                    size
                },
                Err(e) => {
                    // if we got WouldBlock, then this is Non Blocking socket
                    // and data still not available for this, so it's not a connection error
                    if e.kind() == ErrorKind::WouldBlock {
                        return Ok(ret_data);
                    }

                    return Err(e);
                }
            };

            // moving read index foward
            self.read_data_index += read_len;
            // keeping only read data
            data_chunk.truncate(read_len);
            // keeping data in queue
            self.read_data_queue.push(data_chunk);

            if self.read_data_index == self.read_data_len {
                // extracting all data in read queue for this connection
                let mut data_part: Vec<u8> = Vec::new();
                while !self.read_data_queue.is_empty() {
                    data_part.append(&mut self.read_data_queue.remove(0));
                }

                // keeping read data to return
                ret_data.push(Arc::new(data_part));

                // cleanning up for next data
                self.read_data_len = 0;
                self.read_data_index = 0;
            }

            if read_len < potential_read_len {
                break;
            }
        }

        Ok(ret_data)
    }

    pub fn write_data(&mut self) -> Result<bool> {
        Ok(true)
    }
}
