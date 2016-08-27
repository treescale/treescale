extern crate mio;

use mio::tcp::TcpListener;

pub struct TcpServer {
    pub sock: TcpListener
}
