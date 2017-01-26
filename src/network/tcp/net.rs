#![allow(dead_code)]
extern crate mio;

use self::mio::channel::{channel, Sender, Receiver};
use network::{NetworkCommand};
use network::tcp::{TcpReaderCommand};
use network::tcp::{TcpWriterCommand};

/// Structure for handling TCP networking functionality
pub struct TcpNetwork {
    // channel to base networking for transfering commands
    network_channel: Sender<NetworkCommand>,

    // Sender and Receiver for handling commands for Networking
    sender_channel: Sender<TcpNetworkCommand>,
    receiver_channel: Receiver<TcpNetworkCommand>,

    // commands channels for sending data to Reader loops
    reader_channels: Vec<Sender<TcpReaderCommand>>,

    // commands channels for sending data to Writer loops
    writer_channels: Vec<Sender<TcpWriterCommand>>,
}

/// Enumeration for commands available for TcpNetworking
pub enum TcpNetworkCMD {

}

/// Base structure for transferring command over loops to TcpNetworking
pub struct TcpNetworkCommand {

}
