#![allow(dead_code)]
extern crate mio;

use self::mio::{Ready, PollOpt, Token};

use node::{Node, NET_RECEIVER_CHANNEL_TOKEN};
use network::{ConnectionIdentity, Connection, TcpNetwork, SocketType, TcpHandlerCommand, TcpHandlerCMD};
use helper::{Log, NetHelper};
use event::{Event};

use std::error::Error;
use std::process;
use std::sync::Arc;
use std::collections::btree_map::Entry::{Occupied, Vacant};

pub enum NetworkCMD {
    None,
    ConnectionClose,
    HandleConnection,
    HandleEvent
}

pub struct NetworkCommand {
    pub cmd: NetworkCMD,
    pub token: Vec<String>,
    pub value: Vec<u64>,
    pub conn_identity: Vec<ConnectionIdentity>,
    pub event: Vec<Event>
}

pub trait Networking {
    /// Main function to init Networking
    fn init_networking(&mut self);

    /// Handle Networking channel events as a NetworkCommand
    fn notify(&mut self, command: &mut NetworkCommand);

    /// Generating handshake information for sending it over networking handshake
    fn handshake_info(&self) -> Vec<u8>;

    /// main input from event loop to networking
    fn net_ready(&mut self, token: Token, event_kind: Ready) -> bool;

    /// sending event with specific path
    fn emit(&mut self, event: Event);
}


impl NetworkCommand {
    #[inline(always)]
    pub fn new() -> NetworkCommand {
        NetworkCommand {
            cmd: NetworkCMD::None,
            token: vec![],
            value: vec![],
            conn_identity: vec![],
            event: vec![]
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

                let contained_token = match self.connections.entry(token.clone()) {
                    Vacant(entry) => {
                        entry.insert(Connection::new(token.clone(), value, identity));
                        false
                    },
                    Occupied(mut entry) => {
                        // adding connection identity
                        let conn = entry.get_mut();
                        conn.add_identity(identity);                        
                        true
                    },
                };

                if contained_token {
                    // handling new connection
                    self.on_new_connection_channel(&token);
                } else {
                    // if we have API connection
                    if value == 0 {
                        self.on_new_api_connection(&token);
                    } else { // if we have regular Node connection
                        self.on_new_connection(&token, value);
                    }
                }
            }

            NetworkCMD::ConnectionClose => {
                // currently supporting only one connection per single command request
                if command.token.len() != 1 || command.conn_identity.len() != 1 {
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

                // anyway we need to close channel of this connection
                self.on_connection_channel_close(&token);

                // if we need to close full connection
                // letting node know about it
                if remove_conn {
                    self.on_connection_close(&token);
                    self.connections.remove(&token);
                }
            }

            NetworkCMD::HandleEvent => {
                // currently supporting only one connection per single command request
                if command.token.len() != 1 {
                    return;
                }

                // getting token out
                let token = command.token.remove(0);

                while !command.event.is_empty() {
                    let event = command.event.remove(0);
                    // if event processing passing fine
                    // emitting event based on his path
                    if self.on_event_data(&token, &event) && !event.path.is_zero() {
                        // then trying to send event over available connections
                        self.emit(event);
                    }
                }
            }

            NetworkCMD::None => {}
        }
    }

    fn init_networking(&mut self) {
        // Registering Networking receiver
        match self.poll.register(&self.net_receiver_chan
                                 , NET_RECEIVER_CHANNEL_TOKEN
                                 , Ready::readable()
                                 , PollOpt::level()) {
            Ok(_) => {},
            Err(e) => {
                Log::error("Unable to register networking receiver channel to Node POLL service"
                           , e.description());
                process::exit(1);
            }
        }

        self.register_tcp();
    }

    #[inline(always)]
    fn net_ready(&mut self, token: Token, event_kind: Ready) -> bool {
        if token == NET_RECEIVER_CHANNEL_TOKEN {
            // trying to get commands while there is available data
            // if we got error, then data is unavailable
            // and breaking receive loop
            while let Ok(mut cmd) = self.net_receiver_chan.try_recv() {
                self.notify(&mut cmd);
            }

            return true;
        }

        self.tcp_ready(token, event_kind)
    }

    #[inline(always)]
    fn handshake_info(&self) -> Vec<u8> {
        let token_len = self.token.len();
        let total_value_len = token_len + 8;
        // adding 4 byte API version
        // 4 Bytes token string length
        // N bytes for token string
        // 8 bytes for Prime Value
        let mut buffer = vec![0; (4 + 4 + token_len + 8)];
        let mut offset = NetHelper::u32_to_bytes(self.api_version, &mut buffer, 0);
        offset += NetHelper::u32_to_bytes(total_value_len as u32, &mut buffer, offset);
        buffer[offset..offset + token_len].copy_from_slice(self.token.as_bytes());
        offset += token_len;
        NetHelper::u64_to_bytes(self.value, &mut buffer, offset);
        buffer
    }

    #[inline(always)]
    fn emit(&mut self, event: Event) {
        let mut tcp_conns_to_send: Vec<Vec<Token>> = vec![Vec::new(); self.net_tcp_handler_sender_chan.len()];
        let mut event = event;
        for (_, mut conn) in &mut self.connections {
            if conn.value == 0 {
                continue;
            }

            if !event.path.dividable(conn.value) {
                continue;
            }

            // if we trying to send to this connection
            // removing it from path
            event.path.div(conn.value);
            let identity = conn.get_identity();
            match identity.socket_type {
                SocketType::TCP => {
                    tcp_conns_to_send[identity.handler_index].push(identity.socket_token);
                }

                SocketType::NONE => {}
            }
        }

        if tcp_conns_to_send.len() == 0 {
            return;
        }

        let data = Arc::new(match event.to_raw() {
            Some(d) => d,
            None => return
        });

        for i in 0..self.net_tcp_handler_sender_chan.len() {
            if tcp_conns_to_send[i].len() == 0 {
                continue;
            }

            let mut command = TcpHandlerCommand::new();
            command.cmd = TcpHandlerCMD::WriteData;
            command.token = tcp_conns_to_send[i].clone();
            command.data = vec![data.clone()];
            match self.net_tcp_handler_sender_chan[i].send(command) {
                Ok(_) => {},
                Err(e) => {
                    Log::error("Unable to send data to TcpHandler during emiting event", e.description());
                }
            }
        }
    }
}