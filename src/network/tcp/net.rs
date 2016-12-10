#![allow(dead_code)]
extern crate mio;
extern crate num;

use std::sync::{Arc, RwLock};
use std::collections::BTreeMap;
use self::mio::{Token};
use self::mio::channel::{channel, Sender, Receiver};
use network::tcp::{TcpConnection, TcpReaderCommand};
use event::{EventHandlerCommand};

pub enum TcpNetworkCMD {

}

pub struct TcpNetworkCommand {

}

pub struct TcpNetwork {
    // base connections list
    pub connections: Arc<RwLock<BTreeMap<Token, TcpConnection>>>,

    sender_channel: Sender<TcpNetworkCommand>,
    receiver_channel: Receiver<TcpNetworkCommand>,
    // channel for triggering events from networking
    event_handler_channel: Sender<EventHandlerCommand>,
    // vector of channels for sending commands to TcpReaders
    reader_channels: Vec<Sender<TcpReaderCommand>>,
    // basic Round Rubin load balancer index for readers
    reader_channel_index: usize
}

impl TcpNetwork {
    pub fn new(event_channel: Sender<EventHandlerCommand>) -> TcpNetwork {
        let (s, r) = channel::<TcpNetworkCommand>();
        TcpNetwork {
            connections: Arc::new(RwLock::new(BTreeMap::new())),
            reader_channels: Vec::new(),
            event_handler_channel: event_channel,
            sender_channel: s,
            receiver_channel: r,
            reader_channel_index: 0
        }
    }

    pub fn channel(&self) -> Sender<TcpNetworkCommand> {
        self.sender_channel.clone()
    }

    // base function for running TcpNetwork service with TcpReaders
    pub fn run(&mut self, server_port: usize, readers_count: usize) {
        self.reader_channels.reserve(readers_count);
        // TODO: making readers

        // TODO: binding server

        // TODO: starting base event loop
    }

    #[inline(always)]
    fn notify() {

    }

    #[inline(always)]
    fn readable() {

    }

    #[inline(always)]
    fn writeabale() {

    }
}
