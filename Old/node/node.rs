#![allow(dead_code)]
extern crate mio;

use network::Network;
use node::{EventHandler, Event, NodeConfig};

pub struct Node<'a> {
    // Token for current Node
    pub token: String,

    // Prime value for current Node
    pub value: u64,

    // Main networking object for current Node
    pub network: Network<'a>,

    // Event Handler for current Node
    pub event: EventHandler<'a>
}

pub enum  NodeCMD {
    NONE,
    HandleEventData,
}

pub struct NodeCommand {
    pub cmd: NodeCMD,
    pub event: Event
}

impl <'a> Node <'a> {
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
    pub fn start(&'a mut self) {
        self.event.set_node(self);
        self.network.event_handler = vec![&mut self.event];
        self.network.start();
    }

    /// Just a shortcut function for emitting event
    #[inline(always)]
    pub fn emit(&mut self, event: Event) {
        self.network.emit(event)
    }

    /// Just a shortcut function for emitting event to API
    /// Based on specific API tokens, probably load balanced
    #[inline(always)]
    pub fn emit_api(&mut self, event: Event, tokens: Vec<String>) {
        self.network.emit_api(tokens, event)
    }

    /// Just a shortcut function for making TCP client connections
    #[inline(always)]
    pub fn connect_tcp(&mut self, address: &str) {
        self.network.connect_tcp(address);
    }
}
