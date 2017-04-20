mod helper;
mod node;
mod event;
mod network;
mod config;
mod graph;

use node::Node;

fn main() {
    Node::new(&config::parse_args()).start();
}