extern crate mio;
extern crate num;

use self::mio::Token;
use self::num::{BigInt, Zero};
use std::sync::{Arc, RwLock};
use std::str::FromStr;

pub struct Connection {
    pub token: String,
    pub value: BigInt,
    pub api_version: usize,
    pub from_server: bool,
    pub socket_token: Token,
    pub writer_index: usize
}

pub type Connections = Arc<RwLock<Vec<Connection>>>;

impl Connection {
    pub fn new(socket_token: Token, token: String, value: String) -> Connection {
        Connection {
            token: token,
            value: match BigInt::from_str(value.as_str()) {
                Ok(v) => v,
                Err(_) => Zero::zero()
            },
            api_version: 0,
            from_server: true,
            socket_token: socket_token,
            writer_index: 0,
        }
    }
}

pub trait ConnsImpl {
    fn create() -> Connections;
}

impl ConnsImpl for Connections {
    fn create() -> Connections {
        Arc::new(RwLock::new(vec![]))
    }
}