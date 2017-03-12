mod network;
mod helper;
mod node;

fn main() {
    let main_config = node::MainConfig::process_cmd();
    let mut node = node::Node::new(&main_config.node);
    node.start();
}
