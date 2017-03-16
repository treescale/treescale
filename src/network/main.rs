#![allow(dead_code)]
extern crate mio;

use self::mio::{Ready, PollOpt};

use node::{Node, NET_RECEIVER_CHANNEL_TOKEN};
use network::{NetworkCommand, NetworkCMD};
use config::NetworkingConfig;
use helper::{Log, NetHelper};

use std::error::Error;
use std::process;

pub trait Networking {
    /// Main function to init Networking
    fn init(&mut self, config: &NetworkingConfig);

    /// Handle Networking channel events as a NetworkCommand
    fn notify(&mut self, command: &mut NetworkCommand);

    /// Generating handshake information for sending it over networking handshake
    fn handshake_info(&self) -> Vec<u8>;
}

impl Networking for Node {
    #[inline(always)]
    fn notify(&mut self, command: &mut NetworkCommand) {
        match command.cmd {
            NetworkCMD::None => {}
            NetworkCMD::ConnectionClose => {}
            NetworkCMD::HandleConnection => {}
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

    #[inline(always)]
    fn handshake_info(&self) -> Vec<u8> {
        let token_len = self.token.len();
        // adding 4 byte API version
        // 4 Bytes token string length
        // N bytes for token string
        // 8 bytes for Prime Value
        let mut buffer = vec![0; (4 + 4 + token_len + 8)];
        let mut offset = NetHelper::u32_to_bytes(self.api_version, &mut buffer, 0);
        offset += NetHelper::u32_to_bytes(token_len as u32, &mut buffer, offset);
        buffer[offset..offset + token_len].copy_from_slice(self.token.as_bytes());
        offset += token_len;
        NetHelper::u64_to_bytes(self.value, &mut buffer, offset);
        buffer
    }
}