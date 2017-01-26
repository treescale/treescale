#![allow(dead_code)]
extern crate mio;

use self::mio::channel::{channel, Sender, Receiver};
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

}

pub struct TcpReaderCommand {

}
