#![allow(dead_code)]
extern crate num;
extern crate mio;
extern crate byteorder;

use self::num::bigint::BigInt;
use self::num::Zero;
use self::mio::tcp::TcpStream;
use self::mio::Token;
use std::io::{Result, Read, Cursor, Error, ErrorKind};
use self::byteorder::{BigEndian, ReadBytesExt};
use std::os::unix::io::AsRawFd;

const MAX_API_VERSION: usize = 500;

// Maximum length for each message is 30mb
static MAX_NETWORK_MESSAGE_LEN: usize = 30000000;

pub struct TcpConnection {
    pub token: String,
    pub value: BigInt,
    pub api_version: usize,
    pub from_server: bool,
    pub accepted: bool,

    // fields for EventLoop
    pub socket: TcpStream,
    pub socket_token: Token,

    // partial data keepers for handling
    // networking data based on chunks
    pending_data_len: usize,
    pending_data_index: usize,
    pending_data: Vec<u8>,
    // 4 bytes for reading big endian numbers from network
    pending_endian_buf: Vec<u8>,

    // queue for keeping writeabale data
    writae_queue: Vec<Vec<u8>>
}

impl TcpConnection {
    // making new TCP connection based on already created TCP Stream
    // NOTE: we will get connection socket_token from handled connection FD
    // NOTE: based on this, we can't use this method on Windows!!
    pub fn new(sock: TcpStream) -> TcpConnection {
        TcpConnection {
            socket_token: Token(sock.as_raw_fd() as usize),
            socket: sock,

            pending_data_len: 0,
            pending_data_index: 0,
            pending_data: Vec::new(),
            pending_endian_buf: Vec::new(),

            token: String::new(),
            api_version: 0,
            value: Zero::zero(),
            from_server: false,
            accepted: false,

            writae_queue: Vec::new()
        }
    }

    // reading API version on the very beginning and probably inside base TCP networking
    // this will help getting API version first to define how communicate with this connection
    #[inline(always)]
    pub fn read_api_version(&mut self) -> Result<bool> {
        // if we have already data defined bigger than 4 bytes
        // then we need to clean up
        if self.pending_endian_buf.len() >= 4 {
            self.pending_endian_buf.clear();
        }

        let pending_data_len = 4 - self.pending_endian_buf.len();
        let mut version_buf = vec![0; pending_data_len];

        match self.socket.read(&mut version_buf) {
            Ok(length) => {
                self.pending_endian_buf.extend(&version_buf[..length]);
                if self.pending_endian_buf.len() < 4 {
                    // not ready yet for converting Big Endian bytes to API version
                    return Ok(false);
                }

                let mut rdr = Cursor::new(&self.pending_endian_buf);
                self.api_version = rdr.read_u32::<BigEndian>().unwrap() as usize;
                if self.api_version >= MAX_API_VERSION {
                    return Err(Error::new(ErrorKind::InvalidData, "Wrong API version provided"));
                }
            }
            Err(e) => return Err(e)
        }

        // if we got here then we are done with API version reading
        self.pending_endian_buf.clear();
        Ok(true)
    }

    // handling given data from Tcp socket read with by byte chunk
    // so this function will split that data based on Protocol API
    // if this function returns false as a second parameter, then we need to close connection
    // it gave wrong API during data read process
    #[inline(always)]
    pub fn handle_data(&mut self, buffer: &Vec<u8>) -> (Vec<Vec<u8>>, bool) {
        let (buffer_len, mut offset) = (buffer.len(), 0);
        let mut data_chunks: Vec<Vec<u8>> = Vec::new();
        loop {
            let mut still_have = buffer_len - offset;
            if still_have <= 0 {
                break;
            }

            if self.pending_data_len == 0 {
                // cleaning up just in case
                if self.pending_endian_buf.len() >= 4 {
                    self.pending_endian_buf.clear();
                }
                // calculating how many bytes we need to read to complete 4 bytes
                let endian_pending_len = 4 - self.pending_endian_buf.len();
                if still_have < endian_pending_len {
                    self.pending_endian_buf.extend(&buffer[offset..still_have]);
                    break;
                }

                self.pending_endian_buf.extend(&buffer[offset..endian_pending_len]);
                offset += endian_pending_len;
                still_have = buffer_len - offset;

                let mut rdr = Cursor::new(self.pending_endian_buf.clone());
                self.pending_data_len = rdr.read_u32::<BigEndian>().unwrap() as usize;
                self.pending_endian_buf.clear();

                if self.pending_data_len > MAX_NETWORK_MESSAGE_LEN {
                    // notifying to close connection
                    return (vec![], false)
                }

                // allocating buffer for new data
                self.pending_data.reserve(self.pending_data_len);
            }

            let mut copy_buffer_len = self.pending_data_len;
            if still_have < self.pending_data_len {
                copy_buffer_len = still_have;
            }

            // reading data to our pending data
            self.pending_data[self.pending_data_index..(self.pending_data_index + copy_buffer_len)]
                    .copy_from_slice(&buffer[offset..(offset + copy_buffer_len)]);
            offset += copy_buffer_len;
            self.pending_data_index += copy_buffer_len;

            // we got all data which we wanted
            if self.pending_data_len == self.pending_data_len {
                // saving our data as a copy and cleanning pending data
                data_chunks.push(self.pending_data.clone());
                self.pending_data.clear();
                self.pending_data_len = 0;
                self.pending_data_index = 0;
            }
        }

        return (data_chunks, true);
    }
}
