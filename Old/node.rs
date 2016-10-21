extern crate mio;
extern crate num;

use network::tcp_reader::{ReaderLoopCommand, ReaderCommands};
use network::tcp_net::{NetLoopCmd, TcpNetwork};
use network::tcp_client::TcpClient;
use self::num::bigint::{BigInt, Sign};
use self::num::Zero;
use std::sync::Arc;

use mio::Sender;
pub struct Node {
    pub is_api: bool,
    pub token: String,
    pub value: BigInt,
    readers_count: usize,
    readers_chan: Vec<Sender<ReaderLoopCommand>>,
    net_chan: Vec<Sender<NetLoopCmd>>,
}

impl Node {
    pub fn new(is_api: bool, readers_count: usize, value_buf: &[u8]) -> Node {
        let value_b = match BigInt::parse_bytes(value_buf, 10) {
            Some(n) => n,
            None => Zero::zero()
        };

        Node {
            is_api: is_api,
            readers_chan: Vec::new(),
            net_chan: Vec::new(),
            readers_count: readers_count,
            token: String::new(),
            value: value_b
        }
    }

    pub fn run(&mut self, server_address: &str) {
        let (net_chan, readers_chans) = TcpNetwork::run(server_address, self.is_api, self.readers_count);
        self.net_chan.push(net_chan);
        self.readers_chan = readers_chans;
    }

    pub fn write_data(&mut self, path: &[u8], data: Arc<Vec<u8>>) {
        let value_b = match BigInt::parse_bytes(path, 10) {
            Some(n) => n,
            None => Zero::zero()
        };

        for i in 0..self.readers_chan.len() {
            let mut p_v = Vec::new();
            p_v.push(value_b.clone());
            let mut w_d = Vec::new();
            w_d.push(data.clone());
            self.readers_chan[i].send(ReaderLoopCommand {
                cmd: ReaderCommands::WRITE_DATA,
                write_path: p_v,
                conn_socks: Vec::new(),
                write_data: w_d
            });
        }
    }

    pub fn write_str(&mut self, path: &[u8], data: String) {
        self.write_data(path, Arc::new(data.into()));
    }

    pub fn connect(&mut self, address: &str) {
        TcpNetwork::connect(self.net_chan[0].clone(), address);
    }
}
