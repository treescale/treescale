extern crate mio;
extern crate slab;

mod main;
mod handler;
mod conn;

pub use self::main::TcpNetwork;
pub use self::handler::{TcpHandlerCMD, TcpHandlerCommand, TcpHandler};
pub use self::conn::{TcpConnection};

use self::mio::Token;

pub type Slab<T> = slab::Slab<T, Token>;