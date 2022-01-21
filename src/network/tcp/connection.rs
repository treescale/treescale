extern crate mio;

use std::os::unix::io::{IntoRawFd};
use mio::net::{ TcpStream };
use mio::{ Token };

pub struct TcpConnection {
    socket: TcpStream,
    token: Token,
}

impl TcpConnection {
    pub fn new(socket: TcpStream, token: Token) -> TcpConnection {
        TcpConnection {
            socket,
            token,
        }
    }
}
