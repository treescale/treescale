#![allow(dead_code)]
mod net;
mod conn;
mod reader;

pub use self::conn::TcpConnection;
pub use self::net::{TcpNetwork, TcpNetworkCommand, TcpNetworkCMD};
pub use self::reader::{TcpReader, TcpReaderCommand, TcpReaderCMD};


pub const TOKEN_VALUE_SEP: char = '|';
