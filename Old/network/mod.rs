pub mod tcp;
mod conn;
mod net;

pub use network::conn::{Connection, Connections, ConnsImpl, ConnectionType};
pub use network::net::Network;