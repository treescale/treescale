#![allow(dead_code)]
extern crate mio;

use self::mio::channel::{channel, Sender, Receiver};
use network::Connection;

/// Base structure for handling all Networking actions for this project
pub struct Network {
    // List of available connections
    connections: Vec<Connection>,

    // Sender and Receiver for handling commands for Networking
    sender_channel: Sender<NetworkCommand>,
    receiver_channel: Receiver<NetworkCommand>
}

/// Enumeration for commands available for Networking
pub enum NetworkCMD {

}

/// Base structure for transferring command over loops to Networking
pub struct NetworkCommand {

}
