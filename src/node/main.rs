#![allow(dead_code)]
extern crate mio;
extern crate threadpool;
extern crate num_cpus;

use self::mio::{Poll, Events};
use self::mio::channel::{channel, Sender, Receiver};
use self::threadpool::ThreadPool;
use self::mio::tcp::{TcpListener};

use event::{EventCallback, EventCommand, EventHandler};
use network::{NetworkCommand, Connection
              , TcpHandlerCommand, TcpNetwork, Networking
              , Slab, TcpConnection, CONNECTION_COUNT_PRE_ALLOC};
use config::NodeConfig;
use helper::Log;
use node::{EVENT_LOOP_EVENTS_SIZE, EVENT_RECEIVER_CHANNEL_TOKEN};

use std::collections::BTreeMap;
use std::process;
use std::error::Error;

pub struct Node {
    /// Node Valid information for identification
    pub value: u64,
    pub token: String,
    pub api_version: u32,

    /// Callbacks map for handling it in EventHandler trait
    pub callbacks: BTreeMap<String, Vec<EventCallback>>,
    // channels for handling events from networking
    pub event_sender_chan: Sender<EventCommand>,
    pub event_receiver_chan: Receiver<EventCommand>,

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

    /// Thread Pool for making background precessing tasks
    pub thread_pool: ThreadPool,

    /// parent address in case if we are doing something directly from command line
    parent_address: String
}


impl Node {
    /// Making new node based on configurations
    pub fn new(config: &NodeConfig) -> Node {
        let (ev_s, ev_r) = channel::<EventCommand>();
        let (net_s, net_r) = channel::<NetworkCommand>();

        let mut cpu_count = config.network.concurrency;
        if cpu_count == 0 {
            cpu_count = num_cpus::get();
        }

        Node {
            value: config.value,
            token: config.token.clone(),
            api_version: config.api_version,
            callbacks: BTreeMap::new(),
            event_sender_chan: ev_s,
            event_receiver_chan: ev_r,
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
            thread_pool: ThreadPool::new(cpu_count),
            parent_address: config.parent_address.clone()
        }
    }

    /// Starting all services of Node and running event loop
    pub fn start(&mut self) {
        // making networking available
        self.init_networking();
        // making event handlers available
        self.init_event();

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

                // if we have event for EventHandler, implementing it
                if token == EVENT_RECEIVER_CHANNEL_TOKEN {
                    self.event_notify();
                    continue;
                }

                // if this is a networking event just moving to the next event
                // otherwise we will probably check other block implementations
//                if self.net_ready(token, kind) {
//                    continue;
//                }
                self.net_ready(token, kind);
            }
        }
    }
}