#![allow(dead_code)]
use network::Network;
use node::EventHandler;

pub struct Node {
    // Prime value for current Node
    value: u64,

    // Main networking object for current Node
    network: Network,

    // Event Handler for current Node
    event: EventHandler
}
