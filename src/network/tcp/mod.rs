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
pub type Slab<T> = slab::Slab<T, Token>;