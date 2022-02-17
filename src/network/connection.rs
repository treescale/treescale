use mio::Poll;
use network::tcp::TcpConnection;
use std::io;
use std::io::ErrorKind;
use std::net::SocketAddr;

#[derive(PartialEq)]
pub enum ConnectionType {
    Tcp,
}

pub struct Connection<'a> {
    poll: &'a Poll,
    connection_type: ConnectionType,

    // TCP Connection Handlers
    tcp_connection: Option<&'a mut TcpConnection>,
    // We can have UDP or Unix Socket handlers here
}

impl<'a> Connection<'a> {
    pub fn new(poll: &Poll, connection_type: ConnectionType) -> Connection {
        Connection {
            tcp_connection: None,
            poll,
            connection_type,
        }
    }

    pub fn set_tcp_connection(&mut self, connection: &'a mut TcpConnection) {
        self.tcp_connection = Some(connection);
    }

    pub fn write(&mut self, data: Vec<u8>) {
        if self.connection_type == ConnectionType::Tcp && self.tcp_connection.is_some() {
            self.tcp_connection.as_mut().unwrap().write(data, self.poll)
        } else {
            // We can implement UDP or Unix Socket here for example
        }
    }

    pub fn remote_address(&self) -> Result<SocketAddr, io::Error> {
        if self.connection_type == ConnectionType::Tcp && self.tcp_connection.is_some() {
            return self.tcp_connection.as_ref().unwrap().socket.peer_addr();
        }
        Err(io::Error::new(
            ErrorKind::AddrNotAvailable,
            "There is no Connection Socket to get a remote address",
        ))
    }
}
