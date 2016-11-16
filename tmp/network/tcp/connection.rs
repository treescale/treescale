#![allow(dead_code)]
extern crate mio;
extern crate num;
extern crate byteorder;

use self::mio::{Token};
use self::mio::channel::{Sender};
use self::mio::tcp::TcpStream;
use self::num::bigint::BigInt;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write, Result, Error, ErrorKind, Cursor};
use self::byteorder::{BigEndian, ReadBytesExt};
use network::tcp::reader::Reader;

pub struct Connection {
    socket_token: Token,
    pub reader_chan: Sender<Box<Fn(&mut Reader)>>,

    // prime number value for defining path
    value: BigInt,

    // Token for connected node or api
    node_token: String,

    // options for connection type
    is_api: bool,
    from_server: bool,
}


pub struct ReaderConnection {
    // Connection socket handler
    pub socket: TcpStream,

    // fields for reading chunked data
    pub read_chunks: Vec<Vec<u8>>,
    pub read_length: usize,
    pub read_index: usize,

    // token for Event Loop identification
    // this should be set from networking loop
    pub socket_token: Token,

    // Single write queue per connection
    // this would be shared with mutex for thread safety
    pub write_queue: Vec<Vec<u8>>
}

impl ReaderConnection {
    pub fn read_data(&mut self, data_len_container: &mut Vec<u8>, data_container: &mut Vec<u8>) -> Result<(bool, Vec<u8>)> {
        // if we have new data to read
        if self.read_length == 0 {
            match self.socket.read_exact(data_len_container) {
                Ok(()) => {}
                Err(e) => return Err(e)
            };

            let mut rdr = Cursor::new(data_len_container);
            self.read_length = match rdr.read_u32::<BigEndian>() {
                Ok(s) => s,
                Err(e) => return Err(e)
            } as usize;
            self.read_index = 0;
        }

        let read_size = match self.socket.read(data_container) {
            Ok(s) => s,
            Err(e) => return Err(e)
        };

        if read_size <= 0 {
            return Err(Error::new(ErrorKind::Interrupted, "No data received from socket"));
        }

        self.read_index += read_size;
        let (d, _) = data_container.split_at(read_size - 1);
        self.read_chunks.push(Vec::from(d));

        if self.read_index > self.read_length {
            self.read_index = 0;
            self.read_length = 0;
            self.read_chunks.clear();
            return Err(Error::new(ErrorKind::InvalidData, "Received data is larger than expected!"));
        }

        // if we got all data
        if self.read_index == self.read_length {
            let mut ret_data: Vec<u8> = Vec::new();
            // removing and appending data to total vector
            for i in 0..self.read_chunks.len() {
                ret_data.append(&mut self.read_chunks.remove(i));
            }

            self.read_index = 0;
            self.read_length = 0;
            self.read_chunks.clear();
            return Ok((true, ret_data));
        }

        return Ok((false, Vec::new()));
    }

    pub fn flush_data(&mut self) -> Result<bool> {
        while !self.write_queue.is_empty() {
            let write_size = match self.socket.write(self.write_queue[0].as_slice()) {
                Ok(ws) => ws,
                Err(e) => return Err(e)
            };

            if write_size <= 0 || write_size > self.write_queue[0].len() {
                return Ok(false);
            }

            if write_size < self.write_queue[0].len() {
                self.write_queue[0] = self.write_queue[0].split_off(write_size);
                return Ok(false)
            }

            // if we got here then we successfully sent all data
            // so now we need to remove it from list
            self.write_queue.remove(0);
        }

        Ok(true)
    }
}
