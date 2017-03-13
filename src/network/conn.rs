#![allow(dead_code)]
extern crate mio;

pub enum SocketType {
    NONE,
    TCP,
}

pub struct ConnectionIdentity {
    pub writer_index: usize,
    pub socket_type: SocketType
}

pub struct Connection {
    // Connection token for unique Node access
    pub token: String,

    // accepted connection prime value for unique identification
    // and path calculation
    // NOTE: if value is 0 then this connection is API connection
    pub value: u64,

    // List fo connection identities, for giving
    // ability to make multiple connections from single node service
    identities: Vec<ConnectionIdentity>,

    // index for load balancing for write requests
    // over connection identities
    identity_index: usize,

    // is this connection coming from server or client
    pub from_server: bool,
}

impl Connection {
    pub fn new(token: String, value: u64, from_server: bool) -> Connection {
        Connection {
            token: token,
            value: value,
            identities: vec![],
            from_server: from_server,
            identity_index: 0
        }
    }

    /// Checking API version, if it's not correct function will return false
    #[inline(always)]
    pub fn check_api_version(version: u32) -> bool {
        version > 0 && version < 500
    }

    /// Setting Identity for this connection
    /// this will add new identity to existing list
    #[inline(always)]
    pub fn set_identity(&mut self, identity: ConnectionIdentity) {
        self.identities.push(identity);
    }

    /// Load balancing over identities and returning one of them
    /// Returns None is connection don't have any identities
    #[inline(always)]
    pub fn get_identity(&mut self) -> Option<&ConnectionIdentity> {
        if self.identities.len() == 0 {
            return None;
        }

        if self.identity_index >= self.identities.len() {
            self.identity_index = 0;
        }

        let i = self.identity_index;
        self.identity_index += 1;

        Some(&self.identities[i])
    }
}
