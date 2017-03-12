#![allow(dead_code)]
extern crate mio;

use std::error::Error;
use network::Connection;
use network::tcp::TcpNetwork;
use std::collections::BTreeMap;
use self::mio::channel::{Sender, Receiver, channel};
use self::mio::{Token, Poll, Ready, PollOpt, Events};
use network::NetworkConfig;
use helper::{Log, NetHelper};
use std::process;
use std::u32::MAX as u32MAX;

const RECEIVER_CHANNEL_TOKEN: Token = Token((u32MAX - 1) as usize);
const LOOP_EVENTS_COUNT: usize = 64000;
pub type ConnectionsMap = BTreeMap<String, Connection>;

pub enum NetworkCMD {

}

pub struct NetworkCommand {
    cmd: NetworkCMD,
}

pub struct Network {
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
    tcp_net: TcpNetwork
}

impl Network {
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

        Network {
            node_value: value,
            connections: ConnectionsMap::new(),
            tcp_net: TcpNetwork::new(config.server_address.as_str(), config.concurrency, s.clone(), handshake),
            sender_chan: s,
            receiver_chan: r,
            poll: poll,
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

                    // passing event to TCP networking
                    if self.tcp_net.ready(token, kind, &mut self.poll, &mut self.connections) {
                        // if token found in TCP actions moving on
                        continue;
                    }

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
}
