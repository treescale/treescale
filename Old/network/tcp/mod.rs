extern crate slab;
extern crate mio;

mod tcp;
mod conn;
mod reader;
mod writer;

pub use self::conn::{TcpReaderConn, TcpWriterConn};
pub use self::tcp::TcpNetwork;
pub use self::reader::{TcpReader, TcpReaderCommand, TcpReaderCMD};
pub use self::writer::{TcpWriter, TcpWriterCommand, TcpWriterCMD};

use self::mio::Token;
use std::u32::MAX as u32MAX;
pub type Slab<T> = slab::Slab<T, Token>;

pub const CONNECTION_COUNT_PRE_ALLOC: usize = 1024;
pub const SERVER_SOCKET_TOKEN: Token = Token((u32MAX - 2) as usize);