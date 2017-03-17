mod helper;
mod node;
mod event;
mod network;
mod config;

use node::Node;

fn main() {
    Node::new(&config::parse_args()).start();
}