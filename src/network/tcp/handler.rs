#![allow(dead_code)]
extern crate mio;

use network::tcp::TcpConnection;
use network::NetworkCommand;
use event::{EventCommand};

use self::mio::channel::{Sender, Receiver, channel};
use self::mio::{Poll};

pub enum TcpHandlerCMD {
    None,
    HandleConnection,
    WriteData
}

pub struct TcpHandlerCommand {
    pub cmd: TcpHandlerCMD,

}

impl TcpHandlerCommand {
    pub fn new() -> TcpHandlerCommand {
        TcpHandlerCommand {
            cmd: TcpHandlerCMD::None,
        }
    }
}

/// Main struct for handling TCP connections separately for reading and writing
pub struct TcpHandler {
    // Connections for current handler
    connections: Vec<TcpConnection>,

    // channels for TcpHandler
    sender_chan: Sender<TcpHandlerCommand>,
    receiver_chan: Receiver<TcpHandlerCommand>,

    // channel for networking
    net_chan: Sender<NetworkCommand>,

    // channel for event handler
    event_chan: Sender<EventCommand>,

    // poll service for current writer
    poll: Poll,
}