#![allow(dead_code)]
extern crate mio;

use self::mio::Token;

pub enum SocketType {
    TCP,
}

pub struct Connection {
    // accepted connection prime value for unique identification
    // and path calculation
    // NOTE: if value is 0 then this connection is API connection
    value: u64,

    // Socket token for handling socket actions from Slab
    socket_token: Token,
    socket_type: SocketType,

    // is this connection coming from server or client
    from_server: bool,
}

impl Connection {

    /// Checking API version, if it's not correct function will return false
    #[inline(always)]
    pub fn check_api_version(version: u32) -> bool {
        version > 0 && version < 500
    }
}
