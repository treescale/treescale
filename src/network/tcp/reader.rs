#![allow(dead_code)]
extern crate mio;

use self::mio::channel::{channel, Receiver, Sender};
use network::tcp::{Slab, TcpReaderConn, CONNECTION_COUNT_PRE_ALLOC};
use network::NetworkCommand;

pub enum TcpReaderCMD {
    NONE,
    HandleConnection
}

pub struct TcpReaderCommand {
    pub cmd: TcpReaderCMD,
    pub conn: Vec<TcpReaderConn>
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

impl TcpReaderCommand {
    pub fn default() -> TcpReaderCommand {
        TcpReaderCommand {
            cmd: TcpReaderCMD::NONE,
            conn: vec![]
        }
    }
}

impl TcpReader {
    pub fn new(net_chan: Sender<NetworkCommand>) -> TcpReader {
        let (s, r) = channel::<TcpReaderCommand>();
        TcpReader {
            net_chan: net_chan,
            sender_chan: s,
            receiver_chan: r,
            connections: Slab::with_capacity(CONNECTION_COUNT_PRE_ALLOC)
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<TcpReaderCommand> {
        self.sender_chan.clone()
    }

    pub fn start(&mut self) {
        
    }
}