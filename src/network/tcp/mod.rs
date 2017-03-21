extern crate mio;
extern crate slab;

mod main;
mod handler;

pub use self::main::TcpNetwork;
pub use self::handler::{TcpHandlerCMD, TcpHandlerCommand, TcpHandler};
pub use helper::tcp_conn::{TcpConnection};

use self::mio::Token;

pub type Slab<T> = slab::Slab<T, Token>;