#![allow(dead_code)]
mod net;
mod conn;
mod reader;

pub use network::tcp::conn::{TcpConn, TcpConnValue};
pub use network::tcp::reader::{TcpReaderCommand, TcpReaderCMD, TcpReader};

pub const TOKEN_VALUE_SEP: char = '|';
