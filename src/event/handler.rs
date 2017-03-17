#![allow(dead_code)]
extern crate mio;

use self::mio::{PollOpt, Ready};

use node::{Node, EVENT_RECEIVER_CHANNEL_TOKEN};
use event::Event;
use helper::Log;

use std::error::Error;
use std::process;

pub type EventCallback = Box<Fn(&Event, &mut Node) -> bool>;


pub enum EventCMD {
    None,
    HandleEvent
}

pub struct EventCommand {
    pub cmd: EventCMD,
    pub token: Vec<String>,
    pub event: Vec<Event>
}

impl EventCommand {
    #[inline(always)]
    pub fn new() -> EventCommand {
        EventCommand {
            cmd: EventCMD::None,
            token: vec![],
            event: vec![]
        }
    }
}


pub trait EventHandler {
    /// Init event handler
    fn init_event(&mut self);
    /// Adding new callback to event
    /// or adding an event with given name if it's not exists
    fn on(&mut self, name: &str, callback: EventCallback);

    /// Removing event from callbacks list
    fn rm(&mut self, name: &str);

    /// Run callbacks for specific event name
    /// Using given Event object for callback argument
    fn trigger(&mut self, event: &Event);

    /// Function to trigger events from local functions
    fn trigger_local(&mut self, name: &str, from: String, data: Vec<u8>);

    /// handle POLL event and read data from channel
    fn event_notify(&mut self);
}

impl EventHandler for Node {
    fn init_event(&mut self) {
        // Registering Networking receiver
        match self.poll.register(&self.event_receiver_chan
                                 , EVENT_RECEIVER_CHANNEL_TOKEN
                                 , Ready::readable()
                                 , PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                Log::error("Unable to register Event Handler receiver channel to Node POLL service"
                           , e.description());
                process::exit(1);
            }
        }
    }

    #[inline(always)]
    fn on(&mut self, name: &str, callback: EventCallback) {
        let name_str = String::from(name);
        let cbs = match self.callbacks.remove(&name_str) {
            Some(mut callbacks) => {
                callbacks.push(callback);
                callbacks
            }

            None => vec![callback]
        };

        self.callbacks.insert(name_str, cbs);
    }

    #[inline(always)]
    fn rm(&mut self, name: &str) {
        self.callbacks.remove(&String::from(name));
    }

    #[inline(always)]
    fn trigger(&mut self, event: &Event) {
        if !self.callbacks.contains_key(&event.name) {
            return;
        }

        let callbacks = self.callbacks.remove(&event.name).unwrap();
        for cb in &callbacks {
            // if callback returning false then breaking the loop
            if !cb(event, self) {
                break;
            }
        }

        self.callbacks.insert(event.name.clone(), callbacks);
    }

    #[inline(always)]
    fn trigger_local(&mut self, name: &str, from: String, data: Vec<u8>) {
        let mut ev = Event::default();
        ev.from = from;
        ev.name = String::from(name);
        ev.data = data;

        self.trigger(&ev);
    }

    #[inline(always)]
    fn event_notify(&mut self) {
        // trying to get commands while there is available data
        loop {
            match self.event_receiver_chan.try_recv() {
                Ok(cmd) => {
                    let mut command: EventCommand = cmd;
                    match command.cmd {
                        EventCMD::HandleEvent => {
                            while !command.event.is_empty() {
                                // triggering given event
                                self.trigger(&command.event.remove(0));
                            }
                        }
                        EventCMD::None => {}
                    }
                }
                // if we got error, then data is unavailable
                // and breaking receive loop
                Err(e) => {
                    Log::warn("EventHandler receiver channel data is not available",
                              e.description());
                    break;
                }
            }
        }
    }
}