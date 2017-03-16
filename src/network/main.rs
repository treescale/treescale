#![allow(dead_code)]
extern crate mio;

use self::mio::{Ready, PollOpt, Token};

use node::{Node, NET_RECEIVER_CHANNEL_TOKEN};
use network::{ConnectionIdentity, Connection, TcpNetwork, SocketType, TcpHandlerCommand, TcpHandlerCMD};
use config::NetworkingConfig;
use helper::{Log, NetHelper};
use event::{Event, EventHandler};

use std::error::Error;
use std::process;
use std::sync::Arc;

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
    fn init(&mut self, config: &NetworkingConfig);

    /// Handle Networking channel events as a NetworkCommand
    fn notify(&mut self, command: &mut NetworkCommand);

    /// Generating handshake information for sending it over networking handshake
    fn handshake_info(&self) -> Vec<u8>;

    /// main input from event loop to networking
    fn ready(&mut self, token: Token, event_kind: Ready) -> bool;

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

            NetworkCMD::HandleEvent => {
                // currently supporting only one connection per single command request
                if command.event.len() != 1 {
                    return;
                }

                while !command.event.is_empty() {
                    let event = command.event.remove(0);
                    // triggering event if we have it
                    self.trigger(&event);

                    // then trying to send event over available connections
                    self.emit(event);
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

        self.net_tcp_server = Node::make_tcp_server(config.tcp_server_host.as_str());
        self.register_tcp();
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

        let tcp_handler_channels = self.net_tcp_handler_sender_chan.clone();

        self.thread_pool.execute(move || {
            let data = Arc::new(match event.to_raw() {
                Some(d) => d,
                None => return
            });

            for i in 0..tcp_handler_channels.len() {
                if tcp_conns_to_send[i].len() == 0 {
                    continue;
                }

                let mut command = TcpHandlerCommand::new();
                command.cmd = TcpHandlerCMD::WriteData;
                command.token = tcp_conns_to_send[i].clone();
                command.data.push(data.clone());
                match tcp_handler_channels[i].send(command) {
                    Ok(_) => {},
                    Err(e) => {
                        Log::error("Unable to send data to TcpHandler during emiting event", e.description());
                    }
                }
            }
        });
    }
}