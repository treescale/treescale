#![allow(dead_code)]
extern crate mio;

use self::mio::channel::{channel, Receiver, Sender};
use network::tcp::{Slab, TcpWriterConn, CONNECTION_COUNT_PRE_ALLOC};
use network::NetworkCommand;

pub enum TcpWriterCMD {
    NONE,
    HANDLE_CONNECTION,
}

pub struct TcpWriterCommand {
    pub cmd: TcpWriterCMD,
    pub conn: Vec<TcpWriterConn>
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

impl TcpWriterCommand {
    pub fn default() -> TcpWriterCommand {
        TcpWriterCommand {
            cmd: TcpWriterCMD::NONE,
            conn: vec![]
        }
    }
}

impl TcpWriter {
    pub fn new(net_chan: Sender<NetworkCommand>) -> TcpWriter {
        let (s, r) = channel::<TcpWriterCommand>();
        TcpWriter {
            net_chan: net_chan,
            sender_chan: s,
            receiver_chan: r,
            connections: Slab::with_capacity(CONNECTION_COUNT_PRE_ALLOC)
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<TcpWriterCommand> {
        self.sender_chan.clone()
    }

    pub fn start(&mut self) {
        
    }
}