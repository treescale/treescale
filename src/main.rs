extern crate mio;

mod error;
mod network;

use network::tcp_net::{TcpNetwork, NetLoopCmd, LoopCommand};
use network::tcp_conn::TcpConnection;
use std::{thread, time};
use mio::Token;
use mio::tcp::TcpListener;
use std::mem;

fn main() {
    let mut net_chan = TcpNetwork::run("0.0.0.0:8888", false, 2);
    thread::sleep(time::Duration::from_secs(20));
    net_chan.send(NetLoopCmd {
        cmd: LoopCommand::STOP_LOOP,
        token: Token(0),
        address: String::new()
    });

    thread::sleep(time::Duration::from_secs(20));

    println!("Size -> {}", mem::size_of::<TcpConnection>());
}
