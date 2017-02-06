#![allow(dead_code)]
extern crate mio;

use self::mio::channel::{channel, Sender, Receiver};
use self::mio::{Poll, Ready, PollOpt, Events, Token};
use network::{Network, NetworkCommand, NetworkCMD};
use node::Event;
use std::collections::BTreeMap;
use std::thread;
use std::process;

const RECEIVER_CHANNEL_TOKEN: Token = Token(0);

pub struct Node {
    // Events BTreeMap for keeping events and their callbacks
    callbacks: BTreeMap<String, Vec<Box<Fn(&Event, &mut Node) -> bool>>>,

    // Sender and Receiver for handling commands for Node Service
    sender_channel: Sender<NodeCommand>,
    receiver_channel: Receiver<NodeCommand>,

    // poll handler for base Node Service
    poll: Poll
}

pub enum NodeCMD {
    HandleDataEvent,
}

pub struct NodeCommand {
    pub cmd: NodeCMD,
    pub event: Vec<Event>
}

pub struct NodeConfig {
    tcp_address: String,
    concurrency: usize
}

impl Node {
    pub fn new() -> Node {
        let (s, r) = channel::<NodeCommand>();
        Node {
            callbacks: BTreeMap::new(),
            sender_channel: s,
            receiver_channel: r,
            poll: Poll::new().expect("Unable to create Poll service for Base Node service")
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<NodeCommand> {
        self.sender_channel.clone()
    }

    pub fn start(&mut self, conf: NodeConfig) {
        // starting networking and keeping channel
        // for later communication
        let mut network = Network::new(conf.tcp_address.as_str(), self.channel());
        let network_channel = network.channel();
        thread::spawn(move || {
            network.start(conf.concurrency);
        });

        match self.poll.register(&self.receiver_channel, RECEIVER_CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                warn!("Unable to register channel receiver for Node POLL -> {}", e);
                process::exit(1);
            }
        }

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
                                self.notify(&mut c, &network_channel);
                            }
                            // if we got error, then data is unavailable
                            // and breaking receive loop
                            Err(_) => break
                        }
                    }
                    continue;
                }
            }
        }
    }

    #[inline(always)]
    fn notify(&mut self, command: &mut NodeCommand, net_chan: &Sender<NetworkCommand>) {
        match command.cmd {
            NodeCMD::HandleDataEvent => {
                if command.event.len() == 0 {
                    return;
                }

                let event = command.event.remove(0);
                if self.trigger(&event) {
                    let _ = net_chan.send(NetworkCommand{
                        cmd: NetworkCMD::HandleEventData,
                        connection: vec![],
                        event: vec![event]
                    });
                }
            }
        }
    }

    #[inline(always)]
    fn trigger(&mut self, event: &Event) -> bool {
        let mut ret_val = true;
        let cbs = match self.callbacks.remove(&event.name) {
            Some(c) => c,
            None => return ret_val
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
}
