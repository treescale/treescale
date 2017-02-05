#![allow(dead_code)]
extern crate mio;

use self::mio::channel::{channel, Sender, Receiver};
use network::{NetworkCommand};

pub struct Node {
    // Main networking channel for handling
    // Network based operations
    network: Sender<NetworkCommand>,
}
