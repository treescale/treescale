#![allow(dead_code)]
extern crate mio;

use self::mio::Token;
use self::mio::channel::Sender;

pub enum SocketType {
    NONE,
    TCP,
}

pub struct Connection {
    // Connection token for unique Node access
    token: String,

    // accepted connection prime value for unique identification
    // and path calculation
    // NOTE: if value is 0 then this connection is API connection
    value: u64,

    // Socket token for handling socket actions from Slab
    writer_index: usize,
    socket_type: SocketType,

    // is this connection coming from server or client
    from_server: bool,
}

impl Connection {
    pub fn new(token: String, value: u64, wr_index: usize, from_server: bool) -> Connection {
        Connection {
            token: token,
            value: value,
            socket_type: SocketType::NONE,
            from_server: from_server,
            writer_index: wr_index
        }
    }

    /// Checking API version, if it's not correct function will return false
    #[inline(always)]
    pub fn check_api_version(version: u32) -> bool {
        version > 0 && version < 500
    }
}
