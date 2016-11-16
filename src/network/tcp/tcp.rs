extern crate mio;

use network::tcp::TcpConnection;

pub struct TcpNetwork {
    // base connections vector for keeping full networking connections
    pub connections: Vec<TcpConnection>,
}
