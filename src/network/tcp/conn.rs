#![allow(dead_code)]
extern crate mio;
extern crate byteorder;
extern crate num;

use self::mio::{Token};
use self::mio::channel::Sender;
use self::mio::tcp::TcpStream;
use std::sync::Arc;
use std::io::{Result, Read, ErrorKind, Error, Cursor, Write};
use self::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use self::num::Zero;
use self::num::bigint::BigInt;
use network::tcp::{TcpReaderCommand, TcpReaderCMD};

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
    // token for connection socket
    // used for sending data to reader loops
    pub socket_token: Token,

    // prime number value for connected Node path calculation
    // NOTE: if connection is API Client, then value should be 0
    pub value: BigInt,

    // token for connected node
    // this is a unique token sent on a first handshake
    pub token: String,

    pub accepted: bool,
    pub from_server: bool,

    // Network API version number, which will help to keep
    // multiple API level capability
    pub api_version: usize,

    // channel to reader which owns this connection
    pub reader_channel: Sender<TcpReaderCommand>
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

impl TcpWritableData {
    pub fn new(b: Arc<Vec<u8>>) -> TcpWritableData {
        TcpWritableData {
            buf: b,
            offset: MAX_MESSAGE_DATA_LEN
        }
    }
}

impl TcpConnection {
    /// we need reader channel and token to make a new connection object
    /// connection Node Token and BigInt value would be received during first handshake process
    pub fn new(reader_channel: Sender<TcpReaderCommand>, token: Token) -> TcpConnection {
        TcpConnection {
            socket_token: token,
            value: BigInt::zero(),
            token: String::new(),
            accepted: false,
            from_server: false,
            api_version: 0,
            reader_channel: reader_channel
        }
    }

    /// This function sends command to specific Reader to write data
    /// simulating higher level abstraction
    #[inline(always)]
    pub fn write_data(&self, buf: Arc<Vec<u8>>) {
        let _ = self.reader_channel.send(TcpReaderCommand {
            code: TcpReaderCMD::SendData,
            token: vec![self.socket_token],
            socket: Vec::new(),
            data: vec![buf],
        });
    }

    /// Write vector of data at the same time using one call
    /// this would be usefull if we want to send batch data once
    #[inline(always)]
    pub fn write_batch(&self, buf: Vec<Arc<Vec<u8>>>) {
        let _ = self.reader_channel.send(TcpReaderCommand {
            code: TcpReaderCMD::SendData,
            token: vec![self.socket_token],
            socket: Vec::new(),
            data: buf,
        });
    }

    /// Function to send command to reader for closing connection if needed
    #[inline(always)]
    pub fn close(&self) {
        let _ = self.reader_channel.send(TcpReaderCommand {
            code: TcpReaderCMD::CloseConnection,
            token: vec![self.socket_token],
            socket: Vec::new(),
            data: Vec::new(),
        });
    }
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

                if self.read_data_len >= MAX_MESSAGE_DATA_LEN {
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
        while !self.write_queue.is_empty() {
            // writing first part of current data, so we need to write
            // 4 bytes data length as a BigEndian, based on our DATA API
            if self.write_queue[0].offset == 0 {
                let mut data_len_buf = vec![];
                match data_len_buf.write_u32::<BigEndian>(self.write_queue[0].buf.len() as u32) {
                    Ok(_) => {},
                    // if we got error during converting process
                    // of data length to BigEndian, then we have some data error and
                    // removeing this part of a data from write queue
                    Err(_) => {
                        self.write_queue.remove(0);
                        continue;
                    }
                }

                // trying to write all 4 bytes
                // we hope that 4 bytes is very small amount of data to wait
                match self.socket.write_all(&mut data_len_buf) {
                    Ok(wl) => wl,
                    Err(_) => return Ok(false)
                };

                // if we got here then we have written data length BigEndian
                // so setting normal offset value
                self.write_queue[0].offset = 0;
            }
            let b = self.write_queue[0].buf.clone();
            // getting data based on offset
            let (_, wd) = b.split_at(self.write_queue[0].offset);
            let write_len = match self.socket.write(wd) {
                Ok(wl) => wl,
                Err(_) => return Ok(false)
            };

            if write_len < self.write_queue[0].buf.len() {
                self.write_queue[0].offset += write_len;
                return Ok(false);
            }

            // if we got here then data write is done,
            // so we need to remove data from our queue
            self.write_queue.remove(0);
        }

        Ok(true)
    }
}
