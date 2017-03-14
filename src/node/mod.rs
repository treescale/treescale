#![allow(dead_code)]
mod node;
mod event;
mod handler;
mod config;

pub use self::handler::EventHandler;
pub use self::event::Event;
pub use self::node::Node;
pub use self::config::{NodeConfig, MainConfig};

pub const MAX_API_VERSION: u32 = 1000;

/// DEFAULT EVENT NAMES DEFINED
pub const EVENT_ON_NODE_INIT: &'static str = "__||on_node_init||__";
pub const EVENT_ON_CONNECTION_OPEN: &'static str = "__||on_connection_open||__";
pub const EVENT_ON_CONNECTION_CHANNEL_OPEN: &'static str = "__||on_connection_channel_open||__";
pub const EVENT_ON_CONNECTION_CLOSE: &'static str = "__||on_connection_close||__";
pub const EVENT_ON_CONNECTION_CHANNEL_CLOSE: &'static str = "__||on_connection_channel_close||__";
pub const EVENT_ON_NODE_READY: &'static str = "__||on_node_ready||__";
pub const EVENT_ON_API_MESSAGE: &'static str = "__||on_api_message||__";