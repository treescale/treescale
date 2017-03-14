#![allow(dead_code)]
extern crate mio;
extern crate threadpool;

use self::threadpool::ThreadPool;
use std::error::Error;
use network::Connection;
use network::tcp::TcpNetwork;
use std::collections::BTreeMap;
use self::mio::channel::{Sender, Receiver, channel};
use self::mio::{Poll, Ready, PollOpt, Events};
use network::{NetworkConfig, RECEIVER_CHANNEL_TOKEN, LOOP_EVENTS_COUNT, SocketType};
use network::tcp::{TcpWriterCommand, TcpWriterCMD};
use helper::{Log, NetHelper};
use std::process;
use node::{Event, EventHandler, EVENT_ON_NODE_INIT};
use std::sync::Arc;

pub type ConnectionsMap = BTreeMap<String, Connection>;

pub enum NetworkCMD {
    NONE,
    HandleEvent,
    CloseConnection
}

pub struct NetworkCommand {
    pub cmd: NetworkCMD,
    pub token: Vec<String>,
    pub event: Vec<Event>
}

pub struct Network <'a> {
    // value for current Node which will help to send handshake information first
    // All depends on this unique value
    node_value: u64,

    // main collection for connections
    connections: ConnectionsMap,

    // channels for handling Networking command transfer
    sender_chan: Sender<NetworkCommand>,
    receiver_chan: Receiver<NetworkCommand>,

    // poll service for handling events
    poll: Poll,

    // TCP networking
    tcp_net: TcpNetwork<'a>,

    // keeping thread pool here
    thread_pool: ThreadPool,

    pub event_handler: Vec<&'a mut EventHandler<'a>>,
}

impl <'a> Network <'a> {
    pub fn new(value: u64, token: String, config: &NetworkConfig) -> Network {
        let (s, r) = channel::<NetworkCommand>();
        let poll = match Poll::new() {
            Ok(p) => p,
            Err(e) => {
                Log::error("Unable to create POLL service for Networking",
                           e.description());
                process::exit(1);
            }
        };

        // generating handshake information
        let handshake = Network::generate_handshake(value, token.clone(), config.api_version);
        let thread_pool = ThreadPool::new(config.concurrency);
        Network {
            node_value: value,
            connections: ConnectionsMap::new(),
            tcp_net: TcpNetwork::new(config.server_address.as_str(), config.concurrency, s.clone(), handshake, thread_pool.clone()),
            sender_chan: s,
            receiver_chan: r,
            poll: poll,
            thread_pool: thread_pool,
            event_handler: vec![]
        }
    }

    pub fn start(&mut self) {
        match self.poll.register(&self.receiver_chan,
                                 RECEIVER_CHANNEL_TOKEN,
                                 Ready::readable(),
                                 PollOpt::edge()) {
            Ok(_) => {}
            Err(e) => {
                Log::error("Unable to bind Networking receiver channel to POLL I/O service",
                           e.description());
                process::exit(1);
            }
        }

        // registering TCP network server
        self.tcp_net.register(&mut self.poll);

        self.trigger_event(EVENT_ON_NODE_INIT, String::from("local"), vec![]);

        // making events for handling 5K events at once
        let mut events: Events = Events::with_capacity(LOOP_EVENTS_COUNT);
        loop {
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.iter() {
                let (token, kind) = (event.token(), event.kind());
                if token == RECEIVER_CHANNEL_TOKEN {
                    // trying to get commands while there is available data
                    loop {
                        match self.receiver_chan.try_recv() {
                            Ok(cmd) => {
                                let mut c = cmd;
                                self.notify(&mut c);
                            }
                            // if we got error, then data is unavailable
                            // and breaking receive loop
                            Err(e) => {
                                Log::warn("Networking receiver channel data is not available",
                                          e.description());
                                break;
                            }
                        }
                    }
                    continue;
                }

                // passing event to TCP networking
                if self.tcp_net.ready(token, kind, &mut self.poll, &mut self.connections) {
                    // if token found in TCP actions moving on
                    continue;
                }
            }
        }
    }

    #[inline(always)]
    fn notify(&mut self, command: &mut NetworkCommand) {
        match command.cmd {
            _ => {}
        }
    }

    /// Generating Handshake information for sending it with every connection
    #[inline(always)]
    fn generate_handshake(value: u64, token: String, api_version: u32) -> Vec<u8> {
        let (mut offset, text_len) = (0, token.len());
        // making buffer
        // [4 bytes API Number] + [4 bytes text length number] + [text length numbers] + [8 bytes for prime number version]
        let mut buffer: Vec<u8> = vec![0; (4 + 4 + text_len + 8)];
        let text_bytes = token.into_bytes();

        // writing API version
        let mut tmp_offset = NetHelper::u32_to_bytes(api_version, &mut buffer, offset);
        if tmp_offset == 0 {
            Log::error("Unable to write API Version BigEndian number during generating handshake", "returned 0");
            return vec![];
        }
        offset += tmp_offset;

        // Writing text length with more 8 bytes for value
        tmp_offset = NetHelper::u32_to_bytes((text_len + 8) as u32, &mut buffer, offset);
        if tmp_offset == 0 {
            Log::error("Unable to write Token Length BigEndian number during generating handshake", "returned 0");
            return vec![];
        }
        offset += tmp_offset;

        // Adding text bytes
        for i in 0..text_len {
            buffer[offset + i] = text_bytes[i];
        }
        offset += text_len;

        // Writing Prime Value
        tmp_offset = NetHelper::u64_to_bytes(value, &mut buffer, offset);
        if tmp_offset == 0 {
            Log::error("Unable to write Prime Value BigEndian number during generating handshake", "returned 0");
            return vec![];
        }

        buffer
    }


    /// Send event based on Event Path
    #[inline(always)]
    pub fn emit(&mut self, event: Event) {
        let mut tcp_writer_tokens: Vec<Vec<String>> = vec![Vec::new(); self.tcp_net.writer_channels.len()];
        let mut ev = event;
        let mut have_content = false;
        for (token, conn) in &mut self.connections {
            // checking if we have connection inside path
            if !ev.path.dividable(conn.value) {
                continue;
            }

            match conn.get_identity() {
                Some(ref identity) => {
                    match identity.socket_type {
                        SocketType::TCP => {
                            tcp_writer_tokens[identity.writer_index].push(token.clone());
                            have_content = true;
                        }
                        SocketType::NONE => {}
                    }
                },

                /// TODO: we need to close connection if we don't have an identity in it
                None => {}
            };
        }

        if !have_content {
            return;
        }

        self.write_event_pool(tcp_writer_tokens, ev);
    }

    pub fn emit_api(&mut self, api_tokens: Vec<String>, event: Event) {
        let mut tcp_writer_tokens: Vec<Vec<String>> = vec![Vec::new(); self.tcp_net.writer_channels.len()];
        let mut have_content = false;
        for (token, conn) in &mut self.connections {
            for api_token in &api_tokens {
                let str = token.clone();
                let str2 = api_token.clone();
                if str == str2 {
                    match conn.get_identity() {
                        Some(ref identity) => {
                            match identity.socket_type {
                                SocketType::TCP => {
                                    tcp_writer_tokens[identity.writer_index].push(str);
                                    have_content = true;
                                }
                                SocketType::NONE => {}
                            }
                        },

                        /// TODO: we need to close connection if we don't have an identity in it
                        None => {}
                    };
                }
            }
        }

        if !have_content {
            return;
        }

        self.write_event_pool(tcp_writer_tokens, event);
    }

    #[inline(always)]
    fn write_event_pool(&mut self, tokens: Vec<Vec<String>>, ev: Event) {
        let tcp_writer_channels = self.tcp_net.writer_channels.clone();
        let mut tcp_writer_tokens = tokens;
        // if we got here then we have something inside our writer tokens, so we need
        // to parse Event into raw data and send it over writer channels
        self.thread_pool.execute(move || {

            let data = match ev.to_raw() {
                Some(d) => Arc::new(d),
                None => return
            };

            // Sending tcp data
            for i in 0..tcp_writer_tokens.len() {
                if tcp_writer_tokens[i].len() == 0 {
                    continue;
                }

                let mut cmd = TcpWriterCommand::default();
                cmd.cmd = TcpWriterCMD::WriteData;
                cmd.data.push(data.clone());
                cmd.conn_token = tcp_writer_tokens.remove(i);
                let _ = tcp_writer_channels[i].send(cmd);
            }
        });
    }

    /// Making client TCP connection
    #[inline(always)]
    pub fn connect_tcp(&mut self, address: &str) {
        self.tcp_net.connect(address, &mut self.poll);
    }

    #[inline(always)]
    fn trigger_event(&mut self, name: &str, from: String, data: Vec<u8>) {
        if self.event_handler.len() == 0 {
            return;
        }

        self.event_handler[0].trigger_local(name, from, data);
    }
}

impl NetworkCommand {
    pub fn default() -> NetworkCommand {
        NetworkCommand {
            cmd: NetworkCMD::NONE,
            token: vec![],
            event: vec![]
        }
    }
}