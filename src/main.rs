extern crate mio;
extern crate core;

mod constants;
mod helpers;
mod network;

fn main() {
    let mut server = network::TcpServer::new("127.0.0.1:5000");
    server.listen();
}
