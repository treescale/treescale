#![allow(dead_code)]
extern crate mio;

use network::Connection;
use std::collections::BTreeMap;
use self::mio::channel::{Sender, Receiver, channel};
use self::mio::Poll;
use network::NetworkConfig;

pub struct NetworkCommand {

}

pub struct Network {
    // value for current Node which will help to send handshake information first
    // All depends on this unique value
    node_value: u64,

    // main collection for connections
    connections: BTreeMap<u64, Connection>,

    // channels for handling Networking command transfer
    sender_chan: Sender<NetworkCommand>,
    receiver_chan: Receiver<NetworkCommand>,

    // network configuration
    config: NetworkConfig,

    // poll service for handling events
    poll: Poll,
}

impl Network {
    pub fn new(value: u64, config: NetworkConfig) -> Network {
        let (s, r) = channel::<NetworkCommand>();
        Network {
            node_value: value,
            connections: BTreeMap::new(),
            sender_chan: s,
            receiver_chan: r,
            config: config,
            poll: Poll::new().unwrap_or(),
        }
    }

    pub fn start() {}
}