#![allow(dead_code)]
extern crate mio;

use self::mio::channel::{channel, Receiver, Sender};
use network::tcp::{Slab, TcpReaderConn};
use network::NetworkCommand;

pub enum TcpReaderCMD {

}

pub struct TcpReaderCommand {

}

pub struct TcpReader {
    // channels for TcpReader
    sender_chan: Sender<TcpReaderCommand>,
    receiver_chan: Receiver<TcpReaderCommand>,

    // List of connections for working with this TcpReader
    connections: Slab<TcpReaderConn>,

    // channel for base networking/node for sending parsed data to it
    net_chan: Sender<NetworkCommand>
}