#![allow(dead_code)]
extern crate mio;
extern crate log;

use std::collections::BTreeMap;
use event::Event;
use std::sync::Arc;
use std::str::FromStr;
use self::mio::channel::{channel, Receiver, Sender};
use self::mio::{Poll, Token, Ready, PollOpt, Events};
use std::process::exit;
use node::Node;

pub enum EventHandlerCMD {
    TriggerFromEvent
}

pub struct EventHandlerCommand {
    pub cmd: EventHandlerCMD,
    pub event: Arc<Event>
}

const EVENT_CHAN_TOKEN: Token = Token(1);

pub struct EventHandler {
    callbacks: BTreeMap<String, Vec<Box<Fn(Arc<Event>)>>>,
    receiver_channel: Receiver<EventHandlerCommand>,
    sender_chan: Sender<EventHandlerCommand>,
}

impl EventHandler {
    pub fn new() -> EventHandler {
        let (s, r) = channel::<EventHandlerCommand>();
        EventHandler {
            callbacks: BTreeMap::new(),
            receiver_channel: r,
            sender_chan: s,
        }
    }

    pub fn set_node(&mut self) {

    }

    pub fn channel(&self) -> Sender<EventHandlerCommand> {
        self.sender_chan.clone()
    }

    pub fn run(&mut self) {
        let poll = match Poll::new() {
            Ok(p) => p,
            Err(e) => {
                warn!("Unable create Poll object for EventHandler -> {}", e);
                exit(1);
            }
        };

        match poll.register(&self.receiver_channel, EVENT_CHAN_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                warn!("Unable register EventHandler reader channel -> {}", e);
                exit(1);
            }
        };

        let mut events: Events = Events::with_capacity(1000);
        loop {
            let event_count = poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.into_iter() {
                if event.token() == EVENT_CHAN_TOKEN {
                    // trying to get commands while there is available data
                    loop {
                        match self.receiver_channel.try_recv() {
                            Ok(cmd) => {
                                let mut c = cmd;
                                self.notify(&mut c);
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
    fn notify(&mut self, command: &mut EventHandlerCommand) {
        match command.cmd {
            EventHandlerCMD::TriggerFromEvent => {
                match self.callbacks.get(&command.event.name) {
                    Some(cbs) => {
                        for i in 0..cbs.len() {
                            cbs[i](command.event.clone());
                        }
                    }
                    None => {}
                }
            }
        }
    }

    // setting event here
    pub fn on(&mut self, name: &str, callback: Box<Fn(Arc<Event>)>) {
        let mut name_str = String::from_str(name).unwrap();
        if !self.callbacks.contains_key(&mut name_str) {
            self.callbacks.insert(name_str.clone(), Vec::new());
        }

        match self.callbacks.get_mut(&name_str) {
            Some(cbs) => {
                cbs.push(callback);
            }
            None => {}
        }
    }

    // removing event from callbacks
    pub fn remove(&mut self, name: &str) {
        let mut name_str = String::from_str(name).unwrap();
        if !self.callbacks.contains_key(&mut name_str) {
            return;
        }

        self.callbacks.remove(&name_str);
    }
}
