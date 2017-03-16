#![allow(dead_code)]
mod main;
mod conn;
mod tcp;

pub use self::main::{Networking, NetworkCMD, NetworkCommand};
pub use self::conn::{Connection, ConnectionIdentity, SocketType};
pub use self::tcp::{TcpNetwork
                    , TcpHandlerCommand, TcpHandlerCMD
                    , Slab , TcpConnection};

pub const CONNECTION_COUNT_PRE_ALLOC: usize = 1024;