#![allow(dead_code)]
extern crate mio;

use std::sync::{Arc, RwLock};
use std::collections::BTreeMap;
use self::mio::{Token};
use self::mio::channel::{channel, Sender, Receiver};
use network::tcp::{TcpConnection};

pub enum TcpReaderCMD {
    HandleConnection
}

pub struct TcpReaderCommand {
    pub cmd: TcpReaderCMD,
    pub conn: Vec<TcpConnection>
}

pub struct TcpReader {
    // base list of connections,
    // which should be comming from TcpNetworking
    // TcpReader would use this only in read only mode
    connections: Arc<RwLock<BTreeMap<Token, TcpConnection>>>,

    // channels for thread communication
    sender_channel: Sender<TcpReaderCommand>,
    receiver_channel: Receiver<TcpReaderCommand>,
}

impl TcpReader {
    pub fn new(connections: Arc<RwLock<BTreeMap<Token, TcpConnection>>>) -> TcpReader {
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
