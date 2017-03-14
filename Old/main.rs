mod network;
mod helper;
mod node;

use node::{EVENT_ON_CONNECTION_OPEN, EVENT_ON_NODE_INIT, Event, Node};

fn main() {
    let main_config = node::MainConfig::process_cmd();
    let mut node = node::Node::new(&main_config.node);
    let conn_addr = main_config.connect_to.clone();
    node.event.on(EVENT_ON_NODE_INIT, Box::new(move |event: &Event, node: &mut Node| -> bool {
        println!("Node INIT -> {}", node.value);
        if conn_addr.len() > 0 {
            node.connect_tcp(conn_addr.as_str());
        }
        true
    }));

    node.event.on(EVENT_ON_CONNECTION_OPEN, Box::new(|event: &Event, _: &mut Node| -> bool {
        println!("Got NEw connection from -> {}", event.from);
        true
    }));
    node.start();
}
