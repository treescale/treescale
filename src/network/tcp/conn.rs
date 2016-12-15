#![allow(dead_code)]
extern crate num;
extern crate mio;
extern crate byteorder;

use self::num::{BigInt, Zero};
use self::mio::{Token};
use self::mio::tcp::TcpStream;
use std::os::unix::io::AsRawFd;
use std::io::{Result, Read, Cursor, Error, ErrorKind};
use self::byteorder::{BigEndian, ReadBytesExt};
use network::tcp::{TOKEN_VALUE_SEP};
use std::str::FromStr;

const MAX_API_VERSION: usize = 500;
// Maximum length for each message is 30mb
static MAX_NETWORK_MESSAGE_LEN: usize = 30000000;

pub struct TcpConnValue {
    pub token: String,
    pub value: BigInt,
    pub api_version: usize,
    pub from_server: bool,
    pub socket_token: Token
}

pub struct TcpConn {
    pub socket: TcpStream,
    pub socket_token: Token,
    pub api_version: usize,
    pub from_server: bool,

    // partial data keepers for handling
    // networking data based on chunks
    pending_data_len: usize,
    pending_data_index: usize,
    pending_data: Vec<u8>,
    // 4 bytes for reading big endian numbers from network
    pending_endian_buf: Vec<u8>,

    // queue for keeping writeabale data
    pub writae_queue: Vec<Vec<u8>>,

    conn_value: Vec<TcpConnValue>
}

impl TcpConnValue {
    pub fn new(socket_token: Token, token: String, value: String) -> TcpConnValue {
        TcpConnValue {
            token: token,
            value: match BigInt::from_str(value.as_str()) {
                Ok(v) => v,
                Err(_) => Zero::zero()
            },
            api_version: 0,
            from_server: true,
            socket_token: socket_token
        }
    }
}

impl TcpConn {
    pub fn new(sock: TcpStream) -> TcpConn {
        TcpConn {
            // extracting token from already opened connection file descriptor
            // and adding +2 because we have already 0 and 1 tokens reserved, on TcpNetwork side
            // so we don't want to make same token for multiple handles
            socket_token: Token((sock.as_raw_fd() as usize) + 2),

            socket: sock,
            pending_data_len: 0,
            pending_data_index: 0,
            pending_data: vec![],
            pending_endian_buf: vec![],
            writae_queue: vec![],
            api_version: 0,
            from_server: true,
            conn_value: Vec::new()
        }
    }

    #[inline(always)]
    pub fn add_conn_value(&mut self, socket_token: Token, token: String, value: String) {
        let mut tv = TcpConnValue::new(socket_token, token, value);
        tv.api_version = self.api_version;
        tv.from_server = self.from_server;
        self.conn_value.push(tv);
    }

    #[inline(always)]
    pub fn pop_conn_value(&mut self) -> Option<TcpConnValue> {
        self.conn_value.pop()
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
            Err(e) => {
                // if we got WouldBlock, then this is Non Blocking socket
                // and data still not available for this, so it's not a connection error
                if e.kind() == ErrorKind::WouldBlock {
                    return Ok(false);
                }

                return Err(e);
            }
        }

        // if we got here then we are done with API version reading
        self.pending_endian_buf.clear();
        Ok(true)
    }

    // as a first handshake we need to read connection token and prime value
    // this will help to authenticate connection and calculate paths for sending messages
    // if return value is "true" then we got all data, "false" if we need more data to read
    // but socket don't have it at this moment
    // (String, String, bool) - Token, Value, Is Done
    #[inline(always)]
    pub fn read_token_value(&mut self) -> Result<(String, String, bool)> {
        let mut token_str = String::new();
        let mut value_str = String::new();
        if self.pending_data_len == 0 {
            // if we have already data defined bigger than 4 bytes
            // then we need to clean up
            if self.pending_endian_buf.len() >= 4 {
                self.pending_endian_buf.clear();
            }

            let pending_data_len = 4 - self.pending_endian_buf.len();
            let mut buffer_len_buf = vec![0; pending_data_len];
            match self.socket.read(&mut buffer_len_buf) {
                Ok(length) => {
                    self.pending_endian_buf.extend(&buffer_len_buf[..length]);
                    if self.pending_endian_buf.len() < 4 {
                        // not ready yet for converting Big Endian bytes to API version
                        return Ok((token_str, value_str, false));
                    }

                    let mut rdr = Cursor::new(&self.pending_endian_buf);
                    self.pending_data_len = rdr.read_u32::<BigEndian>().unwrap() as usize;
                    if self.pending_data_len >= MAX_NETWORK_MESSAGE_LEN {
                        return Err(Error::new(ErrorKind::InvalidData, "Wrong API version provided"));
                    }
                }
                Err(e) => {
                    // if we got WouldBlock, then this is Non Blocking socket
                    // and data still not available for this, so it's not a connection error
                    if e.kind() == ErrorKind::WouldBlock {
                        return Ok((token_str, value_str, false));
                    }

                    return Err(e);
                }
            }

            // if we got here then we are done with API version reading
            self.pending_endian_buf.clear();
        }

        let need_to_read = self.pending_data_len - self.pending_data_index;
        let mut data_buffer = vec![0; need_to_read];

        match self.socket.read(&mut data_buffer) {
            Ok(rsize) => {
                self.pending_data.extend(&data_buffer[..rsize]);
                self.pending_data_index += rsize;
                if self.pending_data_index < self.pending_data_len {
                    // we need more data to read
                    return Ok((token_str, value_str, false));
                }

                if self.pending_data_index > self.pending_data_len {
                    return Err(Error::new(ErrorKind::InvalidData, "Wrong API version provided"));
                }

                let total_str = String::from_utf8(self.pending_data.clone()).unwrap();
                self.pending_data.clear();
                self.pending_data_len = 0;
                self.pending_data_index = 0;

                let sep_index = match total_str.find(TOKEN_VALUE_SEP) {
                    Some(i) => i,
                    None => return Err(Error::new(ErrorKind::InvalidData, "Wrong API version provided"))
                };

                let (t, v) = total_str.split_at(sep_index);
                token_str = String::from_str(t).unwrap();
                value_str = String::from_str(v).unwrap();
            }
            Err(e) => {
                // if we got WouldBlock, then this is Non Blocking socket
                // and data still not available for this, so it's not a connection error
                if e.kind() == ErrorKind::WouldBlock {
                    return Ok((token_str, value_str, false));
                }

                return Err(e);
            }
        }

        Ok((token_str, value_str, true))
    }

    // handling given data from Tcp socket read with by byte chunk
    // so this function will split that data based on Protocol API
    // if this function returns false as a second parameter, then we need to close connection
    // it gave wrong API during data read process
    #[inline(always)]
    pub fn handle_data(&mut self, buffer: &Vec<u8>, buffer_len: usize) -> (Vec<Vec<u8>>, bool) {
        let mut offset = 0;
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


    pub fn flush_write_queue(&mut self) -> Result<bool> {
        unimplemented!();
        Ok(true)
    }
}
