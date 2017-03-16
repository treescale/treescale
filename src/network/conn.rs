#![allow(dead_code)]
extern crate  mio;

use self::mio::Token;

use node::MAX_API_VERSION;

pub enum SocketType {
    NONE,
    TCP,
}

pub struct ConnectionIdentity {
    pub handler_index: usize,
    pub socket_type: SocketType,
    pub socket_token: Token
}

pub struct Connection {

}

impl Connection {
    /// Checking API version, if it's not correct function will return false
    #[inline(always)]
    pub fn check_api_version(version: u32) -> bool {
        version > 0 && version < MAX_API_VERSION
    }
}