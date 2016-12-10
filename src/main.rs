#[macro_use]
extern crate log;
mod event;
mod node;
mod network;

use std::mem::size_of;
use network::tcp::TcpConnection;

fn main() {
    println!("{}", size_of::<TcpConnection>());
}
