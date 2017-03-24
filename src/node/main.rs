#![allow(dead_code)]
extern crate mio;
extern crate num_cpus;
extern crate uuid;

use self::mio::{Poll, Events};
use self::mio::channel::{channel, Sender, Receiver};
use self::mio::tcp::{TcpListener};

use network::{NetworkCommand, Connection
              , TcpHandlerCommand, TcpNetwork, Networking
              , Slab, TcpConnection, CONNECTION_COUNT_PRE_ALLOC};
use config::NodeConfig;
use helper::Log;
use node::{EVENT_LOOP_EVENTS_SIZE, DEFAULT_API_VERSION};
use event::Event;

use std::collections::BTreeMap;
use std::process;
use std::error::Error;

pub struct Node {
    /// Node Valid information for identification
    pub value: u64,
    pub token: String,
    pub api_version: u32,

    /// Members for Network trait
    pub connections: BTreeMap<String, Connection>,
    pub net_sender_chan: Sender<NetworkCommand>,
    pub net_receiver_chan: Receiver<NetworkCommand>,

    /// TCP networking params
    pub net_tcp_handler_sender_chan: Vec<Sender<TcpHandlerCommand>>,
    // index for load balancing over TCP Reader and Writer channels
    pub net_tcp_handler_index: usize,
    // TCP server socket
    pub net_tcp_server: TcpListener,
    // keeping just a simple TcpConnection as a pending connection
    pub net_tcp_pending_connections: Slab<TcpConnection>,

    /// POLL service for this node thread event loop
    pub poll: Poll,

    /// parent address in case if we are doing something directly from command line
    parent_address: String
}


impl Node {
    /// Making new node based on configurations
    pub fn new(config: &NodeConfig) -> Node {
        let (net_s, net_r) = channel::<NetworkCommand>();

        let mut cpu_count = config.network.concurrency;
        if cpu_count == 0 {
            cpu_count = num_cpus::get();
        }

        Node {
            value: config.value,
            token: if config.token.len() == 0 { format!("{}", uuid::Uuid::new_v4()) } else { config.token.clone() },
            api_version: if config.api_version == 0 { DEFAULT_API_VERSION } else { config.api_version },
            connections: BTreeMap::new(),
            net_sender_chan: net_s,
            net_receiver_chan: net_r,
            net_tcp_handler_sender_chan: Vec::with_capacity(cpu_count),
            net_tcp_handler_index: 0,
            net_tcp_server: Node::make_tcp_server(config.network.tcp_server_host.as_str()),
            net_tcp_pending_connections: Slab::with_capacity(CONNECTION_COUNT_PRE_ALLOC),
            poll: match Poll::new() {
                Ok(p) => p,
                Err(e) => {
                    Log::error("Unable to create POLL service for Node", e.description());
                    process::exit(1);
                }
            },
            parent_address: config.parent_address.clone()
        }
    }

    /// Starting all services of Node and running event loop
    pub fn start(&mut self) {
        // making networking available
        self.init_networking();

        if self.parent_address.len() > 0 {
            let address = self.parent_address.clone();
            self.tcp_connect(address.as_str());
        }

        // starting base event loop
        // making events for handling 5K events at once
        let mut events: Events = Events::with_capacity(EVENT_LOOP_EVENTS_SIZE);
        loop {
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.iter() {
                let (token, kind) = (event.token(), event.kind());

                // if this is a networking event just moving to the next event
                // otherwise we will probably check other block implementations
//                if self.net_ready(token, kind) {
//                    continue;
//                }
                self.net_ready(token, kind);
            }
        }
    }

    /// Handling new connection here
    pub fn on_new_connection(&mut self, token: &String, value: u64) {
        println!("Got New Connection -> {} {}", token, value);
    }

    /// Handling new API connection here
    pub fn on_new_api_connection(&mut self, token: &String) {
        println!("Got New API Connection -> {}", token);
    }

    /// Handling new identity/channel from existing connection
    pub fn on_new_connection_channel(&mut self, token: &String) {
        println!("Got New Connection Channel -> {}", token);
    }

    /// Handling Connection Close Functionality
    pub fn on_connection_close(&mut self, token: &String) {
        println!("Connection Closed -> {}", token);
    }

    /// Handling Connection Close Functionality
    pub fn on_connection_channel_close(&mut self, token: &String) {
        println!("Connection Channel Closed -> {}", token);
    }

    /// Handling data/event from connection
    /// if this function returns "false" then we wouldn't make any emit process for this event
    /// if this function returns "true" we will continue emitting this evenT
    #[inline(always)]
    pub fn on_event_data(&mut self, token: &String, event: &Event) -> bool {
//        println!("Got data from connection -> {} -> {}", token, event.from);
        true
    }
}