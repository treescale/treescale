#![allow(dead_code)]
extern crate mio;

use self::mio::Token;
use self::mio::channel::{Sender};
use network::tcp::{TcpWriterCommand, TcpReaderConn};

pub enum ConnectionType {
    TCP
}

/// Base Connection structure for handling base information of connection
pub struct Connection {
    pub value: u64,
    pub socket_token: Token,
    pub api_version: u32,
    pub conn_type: ConnectionType,
    pub from_server: bool,

    // writer command for TCP connection or None if this is not a TCP connection
    tcp_writer_chan: Option<Sender<TcpWriterCommand>>,
}

impl Connection {
    pub fn from_tcp(tcp_conn: &TcpReaderConn, writer: Sender<TcpWriterCommand>, from_server: bool) -> Connection {
        Connection {
            value: tcp_conn.value,
            socket_token: tcp_conn.socket_token,
            api_version: tcp_conn.api_version,
            conn_type: ConnectionType::TCP,
            from_server: from_server,
            tcp_writer_chan: Some(writer)
        }
    }
}
