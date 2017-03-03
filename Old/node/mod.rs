mod node;
mod event;


/// Default event names list
pub const EVENT_NODE_INIT: &'static str = "on_node_init";
pub const EVENT_ON_CONNECTION_OPEN: &'static str = "on_connection_open";
// pub const EVENT_ON_CONNECTION_CLOSE: &'static str = "on_connection_close";

pub use self::node::{Node, NodeCommand, NodeCMD, EventCallback, NodeConfig};
pub use self::event::Event;
