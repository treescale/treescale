extern crate mio;
extern crate slab;

mod main;
mod reader;
mod writer;
mod conn;

pub use self::main::TcpNetwork;
pub use self::reader::{TcpReaderCMD, TcpReaderCommand};
pub use self::writer::{TcpWriterCMD, TcpWriterCommand};
pub use self::conn::{TcpConnection};

use self::mio::Token;

pub type Slab<T> = slab::Slab<T, Token>;