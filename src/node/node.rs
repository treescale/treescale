#![allow(dead_code)]
extern crate mio;

use network::Network;
use node::{EventHandler, Event, NodeConfig};

pub struct Node {
    // Token for current Node
    pub token: String,

    // Prime value for current Node
    pub value: u64,

    // Main networking object for current Node
    pub network: Network,

    // Event Handler for current Node
    pub event: EventHandler,
}

pub enum  NodeCMD {
    NONE,
    HandleEventData,
}

pub struct NodeCommand {
    pub cmd: NodeCMD,
    pub event: Event
}

impl Node {
    /// Making New Node Service
    pub fn new(config: &NodeConfig) -> Node {
        Node {
            value: config.value,
            token: config.token.clone(),
            network: Network::new(config.value, config.token.clone(), &config.network),
            event: EventHandler::new()
        }
    }

    /// Starting Node by starting networking
    pub fn start(&mut self) {
        self.network.start();
    }
}
