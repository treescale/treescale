#![allow(dead_code)]
extern crate mio;

use self::mio::{Ready, PollOpt, Token};

use node::{Node, NET_RECEIVER_CHANNEL_TOKEN};
use network::{ConnectionIdentity, Connection, TcpNetwork};
use config::NetworkingConfig;
use helper::{Log, NetHelper};

use std::error::Error;
use std::process;

pub enum NetworkCMD {
    None,
    ConnectionClose,
    HandleConnection
}

pub struct NetworkCommand {
    pub cmd: NetworkCMD,
    pub token: Vec<String>,
    pub value: Vec<u64>,
    pub conn_identity: Vec<ConnectionIdentity>
}

pub trait Networking {
    /// Main function to init Networking
    fn init(&mut self, config: &NetworkingConfig);

    /// Handle Networking channel events as a NetworkCommand
    fn notify(&mut self, command: &mut NetworkCommand);

    /// Generating handshake information for sending it over networking handshake
    fn handshake_info(&self) -> Vec<u8>;

    /// main input from event loop to networking
    fn ready(&mut self, token: Token, event_kind: Ready) -> bool;
}


impl NetworkCommand {
    pub fn new() -> NetworkCommand {
        NetworkCommand {
            cmd: NetworkCMD::None,
            token: vec![],
            value: vec![],
            conn_identity: vec![]
        }
    }
}

impl Networking for Node {
    #[inline(always)]
    fn notify(&mut self, command: &mut NetworkCommand) {
        match command.cmd {
            NetworkCMD::HandleConnection => {
                // currently supporting only one connection per single command request
                if command.token.len() != 1
                    || command.conn_identity.len() != 1
                    || command.value.len() != 1 {
                    return;
                }

                let token = command.token.remove(0);
                let identity = command.conn_identity.remove(0);
                let value = command.value.remove(0);

                if !self.connections.contains_key(&token) {
                    self.connections.insert(token.clone(), Connection::new(token.clone(), value, identity));
                } else {
                    // adding connection identity
                    match self.connections.get_mut(&token) {
                        Some(conn) => {
                            conn.add_identity(identity);
                        }
                        None => {}
                    }
                }
            }
            NetworkCMD::ConnectionClose => {
                // currently supporting only one connection per single command request
                if command.token.len() != 1 {
                    return;
                }

                let token = command.token.remove(0);
                let identity = command.conn_identity.remove(0);
                let remove_conn = match self.connections.get_mut(&token) {
                    Some(conn) => {
                        conn.rm_identity(identity.socket_token, identity.handler_index);
                        // if identity count is 0, we need to close connection
                        conn.identity_count() == 0
                    },
                    None => return
                };

                if remove_conn {
                    self.connections.remove(&token);
                }
            }
            NetworkCMD::None => {}
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
    fn ready(&mut self, token: Token, event_kind: Ready) -> bool {
        self.tcp_ready(token, event_kind)
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