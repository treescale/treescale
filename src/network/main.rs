#![allow(dead_code)]
extern crate mio;

use self::mio::{Ready, PollOpt};

use node::{Node, NET_RECEIVER_CHANNEL_TOKEN};
use network::{NetworkCommand, NetworkCMD};
use config::NetworkingConfig;
use helper::Log;

use std::error::Error;
use std::process;

pub trait Networking {
    /// Main function to init Networking
    fn init(&mut self, config: &NetworkingConfig);

    /// Handle Networking channel events as a NetworkCommand
    fn notify(&mut self, command: &mut NetworkCommand);
}

impl Networking for Node {
    #[inline(always)]
    fn notify(&mut self, command: &mut NetworkCommand) {
        match command.cmd {
            NetworkCMD::None => {}
            NetworkCMD::ConnectionClose => {}
        }
    }

    fn init(&mut self, config: &NetworkingConfig) {
        // Registering Networking receiver
        match self.poll.register(&self.net_receiver_chan
                                 , NET_RECEIVER_CHANNEL_TOKEN
                                 , Ready::readable()
                                 , PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                Log::error("Unable to register networking receiver channel to Node POLL service"
                           , e.description());
                process::exit(1);
            }
        }
    }
}