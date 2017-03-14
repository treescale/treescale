mod network;
mod helper;
mod node;

use node::{EVENT_ON_CONNECTION_OPEN, Event, Node};

fn main() {
    let main_config = node::MainConfig::process_cmd();
    let mut node = node::Node::new(&main_config.node);
    node.event.on(EVENT_ON_CONNECTION_OPEN, Box::new(|event: &Event, _: &mut Node| -> bool {
        println!("Got NEw connection from -> {}", event.from);
        true
    }));
    node.start();
}
