#![allow(dead_code)]
extern crate mio;

use self::mio::channel::{channel, Receiver, Sender};
use network::tcp::{Slab, TcpWriterConn};
use network::NetworkCommand;

pub enum TcpWriterCMD {

}

pub struct TcpWriterCommand {

}

pub struct TcpWriter {
    // channels for TcpReader
    sender_chan: Sender<TcpWriterCommand>,
    receiver_chan: Receiver<TcpWriterCommand>,

    // List of connections for working with this TcpReader
    connections: Slab<TcpWriterConn>,

    // channel for base networking/node for sending parsed data to it
    net_chan: Sender<NetworkCommand>
}