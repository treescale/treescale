#![allow(dead_code)]
extern crate  mio;

use self::mio::Token;

use config::MAX_API_VERSION;

#[derive(Clone)]
pub enum SocketType {
    NONE,
    TCP,
}

#[derive(Clone)]
pub struct ConnectionIdentity {
    pub handler_index: usize,
    pub socket_type: SocketType,
    pub socket_token: Token
}

pub struct Connection {
    /// token for this connection
    pub token: String,

    /// Prime value for this connection
    pub value: u64,

    /// list of identities for this connection
    /// it's basically streams to support data transfer
    /// attached to current connection
    identities: Vec<ConnectionIdentity>,

    /// index for making round rubin for writing data
    /// over identities for this connection
    identity_index: usize
}

impl Connection {
    /// Making new connection with token, value and identity
    /// Connection should have min. 1 identity
    #[inline(always)]
    pub fn new(token: String, value: u64, identity: ConnectionIdentity) -> Connection {
        Connection {
            token: token,
            value: value,
            identities: vec![identity],
            identity_index: 0
        }
    }

    #[inline(always)]
    pub fn add_identity(&mut self, identity: ConnectionIdentity) {
        self.identities.push(identity);
    }

    #[inline(always)]
    pub fn rm_identity(&mut self, socket_token: Token, index: usize) {
        for i in 0..self.identities.len() {
            if self.identities[i].handler_index == index && self.identities[i].socket_token == socket_token {
                self.identities.remove(i);
                return;
            }
        }
    }

    #[inline(always)]
    pub fn identity_count(&self) -> usize {
        self.identities.len()
    }

    pub fn get_identity(&mut self) -> ConnectionIdentity {
        if self.identity_index >= self.identities.len() {
            self.identity_index = 0;
        }

        let i = self.identity_index;
        self.identity_index += 1;
        self.identities[i].clone()
    }

    /// Checking API version, if it's not correct function will return false
    #[inline(always)]
    pub fn check_api_version(version: u32) -> bool {
        version > 0 && version < MAX_API_VERSION
    }
}