#![allow(dead_code)]
extern crate mio;

use self::mio::channel::{channel, Sender, Receiver};
use self::mio::{Poll, Ready, PollOpt, Events, Token};
use network::{Network, NetworkCommand, NetworkCMD};
use node::{Event, EVENT_NODE_INIT};
use std::collections::BTreeMap;
use std::thread;
use std::process;

const RECEIVER_CHANNEL_TOKEN: Token = Token(0);
const CURRENT_API_VERSION: u32 = 1;
pub type EventCallback = Box<Fn(&Event, &mut Node) -> bool>;

pub struct Node {
    // Events BTreeMap for keeping events and their callbacks
    callbacks: BTreeMap<String, Vec<EventCallback>>,

    // Sender and Receiver for handling commands for Node Service
    sender_channel: Sender<NodeCommand>,
    receiver_channel: Receiver<NodeCommand>,

    // channel for networking
    net_chan: Vec<Sender<NetworkCommand>>,

    // poll handler for base Node Service
    poll: Poll,

    // keeping current node API version for sending during client requests
    current_api_version: u32,

    // keeping current node Value for sending it during client requests
    pub current_value: u64,
}

pub enum NodeCMD {
    HandleDataEvent,
}

pub struct NodeCommand {
    pub cmd: NodeCMD,
    pub event: Vec<Event>,
}

pub struct NodeConfig {
    pub tcp_address: String,
    pub concurrency: usize,
}

impl Node {
    pub fn new(current_value: u64) -> Node {
        let (s, r) = channel::<NodeCommand>();
        Node {
            callbacks: BTreeMap::new(),
            sender_channel: s,
            receiver_channel: r,
            poll: Poll::new().expect("Unable to create Poll service for Base Node service"),
            net_chan: vec![],
            current_value: current_value,
            current_api_version: CURRENT_API_VERSION,
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<NodeCommand> {
        self.sender_channel.clone()
    }

    pub fn start(&mut self, conf: NodeConfig) {
        // starting networking and keeping channel
        // for later communication
        let mut network = Network::new(conf.tcp_address.as_str(),
                                       self.channel(),
                                       self.current_api_version,
                                       self.current_value);
        self.net_chan = vec![network.channel()];
        thread::spawn(move || {
            network.start(conf.concurrency);
        });

        match self.poll.register(&self.receiver_channel,
                                 RECEIVER_CHANNEL_TOKEN,
                                 Ready::readable(),
                                 PollOpt::edge()) {
            Ok(_) => {}
            Err(e) => {
                warn!("Unable to register channel receiver for Node POLL -> {}", e);
                process::exit(1);
            }
        }

        // Sending event about node init!
        self.init_event();

        // making events for handling 5K events at once
        let mut events: Events = Events::with_capacity(5000);
        loop {
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.iter() {
                if event.token() == RECEIVER_CHANNEL_TOKEN {
                    // trying to get commands while there is available data
                    loop {
                        match self.receiver_channel.try_recv() {
                            Ok(cmd) => {
                                let mut c = cmd;
                                self.notify(&mut c);
                            }
                            // if we got error, then data is unavailable
                            // and breaking receive loop
                            Err(_) => break,
                        }
                    }
                    continue;
                }
            }
        }
    }

    #[inline(always)]
    fn init_event(&self) {
        let mut e = Event::default();
        e.name = String::from(EVENT_NODE_INIT);
        e.target = String::from("local");

        let _ = self.sender_channel.send(NodeCommand {
            cmd: NodeCMD::HandleDataEvent,
            event: vec![e],
        });
    }

    #[inline(always)]
    fn notify(&mut self, command: &mut NodeCommand) {
        match command.cmd {
            NodeCMD::HandleDataEvent => {
                if command.event.len() == 0 {
                    return;
                }

                let event = command.event.remove(0);
                if self.trigger(&event) {
                    self.emit(event);
                }
            }
        }
    }

    #[inline(always)]
    pub fn trigger(&mut self, event: &Event) -> bool {
        let mut ret_val = true;
        let cbs = match self.callbacks.remove(&event.name) {
            Some(c) => c,
            None => return ret_val,
        };

        for cb in &cbs {
            if !cb(event, self) {
                ret_val = false;
                break;
            }
        }

        self.callbacks.insert(event.name.clone(), cbs);
        ret_val
    }

    #[inline(always)]
    pub fn on(&mut self, name: &str, callback: EventCallback) {
        let name_str = String::from(name);
        match self.callbacks.get_mut(&name_str) {
            Some(cbs) => {
                cbs.push(callback);
                return;
            }
            None => {}
        };

        self.callbacks.insert(name_str.clone(), vec![callback]);
    }

    #[inline(always)]
    pub fn remove(&mut self, name: &str) {
        let name_str = String::from(name);
        // just removing without checking
        self.callbacks.remove(&name_str);
    }

    #[inline(always)]
    pub fn emit(&self, event: Event) {
        if self.net_chan.len() == 0 {
            return;
        }

        let _ = self.net_chan[0].send(NetworkCommand {
            cmd: NetworkCMD::HandleEventData,
            connection: vec![],
            event: vec![event],
            client_address: String::new(),
        });
    }

    #[inline(always)]
    pub fn tcp_connect(&self, address: &str) {
        if self.net_chan.len() == 0 {
            warn!("We don't have active networking service to make client connection");
            return;
        }

        let _ = self.net_chan[0].send(NetworkCommand {
            cmd: NetworkCMD::TCPClientConnection,
            connection: vec![],
            event: vec![],
            client_address: String::from(address),
        });
    }
}
