#![allow(dead_code)]
extern crate mio;
extern crate num;
extern crate byteorder;

use self::mio::{Token};
use self::mio::tcp::TcpStream;
use self::num::bigint::BigInt;
use std::sync::{Arc, Mutex};
use std::io::{Read, Result, Error, ErrorKind, Cursor};
use self::byteorder::{BigEndian, ReadBytesExt};

pub struct Connection {
    node_token: String,
    reader_index: usize,
    reader_token: Token,
    value: BigInt,

    // reader connection mutex for accessing write queue
    reader_conn: Arc<Mutex<ReaderConnection>>
}

pub struct ReaderConnection {
    socket: TcpStream,
    conn_token: Token,

    // partial read variables
    read_chunks: Vec<Vec<u8>>,
    read_length: usize,
    read_index: usize,
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
}
