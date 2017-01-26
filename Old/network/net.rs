#![allow(dead_code)]
use network::{Connections, ConnsImpl};
use network::tcp::TcpNetwork;

pub struct Network {
    // base connections for current Networking
    connections: Connections,
    // main object for TCP networking
//    tcp_net: TcpNetwork
}

impl Network {
    pub fn new() -> Network {
        Network {
            connections: Connections::create(),
        }
    }
}