extern crate mio;

use network::tcp::{TcpConnValue, TcpConn};
use std::sync::{Arc, RwLock};
use self::mio::channel::{Sender, Receiver, channel};

pub enum TcpReaderCMD {
    HandleConnection
}

pub struct TcpReaderCommand {
    pub cmd: TcpReaderCMD,
    pub conn_value: Vec<TcpConnValue>,
    pub conn: Vec<TcpConn>
}

pub struct TcpReader {
    connections: Arc<RwLock<Vec<TcpConnValue>>>,

    // reader sender channels
    sender_channel: Sender<TcpReaderCommand>,
    receiver_channel: Receiver<TcpReaderCommand>
}

impl TcpReader {
    pub fn new(connections: Arc<RwLock<Vec<TcpConnValue>>>) -> TcpReader {
        let (s, r) = channel::<TcpReaderCommand>();
        TcpReader {
            connections: connections,
            sender_channel: s,
            receiver_channel: r
        }
    }

    pub fn channel(&self) -> Sender<TcpReaderCommand> {
        self.sender_channel.clone()
    }

    pub fn run(&mut self) {

    }
}
