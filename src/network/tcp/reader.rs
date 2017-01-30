#![allow(dead_code)]
extern crate mio;

use self::mio::channel::{channel, Sender, Receiver};
use self::mio::tcp::TcpStream;
use network::tcp::{TcpNetworkCommand};
use network::{NetworkCommand};

/// Using this struct we are reading data from TCP connection sockets
pub struct TcpReader {
    // TcpNetworking channel for sending commands to it
    pub tcp_net_channel: Sender<TcpNetworkCommand>,

    // Channel to base Networking for passing commands to it
    pub network_channel: Sender<NetworkCommand>,

    // Sender and Receiver for handling commands for TcpReader
    sender_channel: Sender<TcpReaderCommand>,
    receiver_channel: Receiver<TcpReaderCommand>,
}

pub enum TcpReaderCMD {
    HandleNewConnection,
}

pub struct TcpReaderCommand {
    pub cmd: TcpReaderCMD,
    pub socket: Option<TcpStream>
}

impl TcpReader {
    pub fn new(tcp_net: Sender<TcpNetworkCommand>, net: Sender<NetworkCommand>) -> TcpReader {
        let (s, r) = channel::<TcpReaderCommand>();
        TcpReader {
            tcp_net_channel: tcp_net,
            network_channel: net,
            sender_channel: s,
            receiver_channel: r
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<TcpReaderCommand> {
        self.sender_channel.clone()
    }

    pub fn start(&mut self) {

    }
}
