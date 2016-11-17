#![allow(dead_code)]
extern crate mio;

use network::tcp::TcpConnection;
use self::mio::{Token};
use std::sync::Arc;

pub enum TcpNetworkCMD {
    ConnectionClosed,
    HandleNewData,
}

pub struct TcpNetworkCommand {
    pub cmd: TcpNetworkCMD,
    pub token: Token,
    pub data: Vec<Arc<Vec<u8>>>
}

pub struct TcpNetwork {
    // base connections vector for keeping full networking connections
    pub connections: Vec<TcpConnection>,
}
