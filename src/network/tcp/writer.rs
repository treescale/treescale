#![allow(dead_code)]
extern crate mio;

use self::mio::channel::{channel, Sender, Receiver};
use network::tcp::{TcpNetworkCommand, TcpWriterConn};
use network::{NetworkCommand};


pub struct TcpWriter {
    // TcpNetworking channel for sending commands to it
    pub tcp_net_channel: Sender<TcpNetworkCommand>,

    // Channel to base Networking for passing commands to it
    pub network_channel: Sender<NetworkCommand>,

    // Sender and Receiver for handling commands for TcpReader
    sender_channel: Sender<TcpWriterCommand>,
    receiver_channel: Receiver<TcpWriterCommand>,
}

pub enum TcpWriterCMD {
    HandleNewConnection,
}

pub struct TcpWriterCommand {
    pub cmd: TcpWriterCMD,
    pub conn: Vec<TcpWriterConn>
}

impl TcpWriter {
    pub fn new(tcp_net: Sender<TcpNetworkCommand>, net: Sender<NetworkCommand>) -> TcpWriter {
        let (s, r) = channel::<TcpWriterCommand>();
        TcpWriter {
            tcp_net_channel: tcp_net,
            network_channel: net,
            sender_channel: s,
            receiver_channel: r
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<TcpWriterCommand> {
        self.sender_channel.clone()
    }

    pub fn start(&mut self) {

    }
}
