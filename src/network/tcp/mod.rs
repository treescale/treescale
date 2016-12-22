#![allow(dead_code)]
mod net;
mod conn;
mod reader;

pub use network::tcp::conn::TcpConn;
pub use network::tcp::reader::{TcpReaderCommand, TcpReaderCMD, TcpReader};
pub use network::tcp::net::{TcpNetwork, TcpNetworkCMD, TcpNetworkCommand};

pub const TOKEN_VALUE_SEP: char = '|';
