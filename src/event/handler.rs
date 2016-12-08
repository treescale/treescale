#![allow(dead_code)]
#![allow(unreachable_code)]
extern crate mio;

use self::mio::{Poll, Token, Ready, PollOpt, Events};
use self::mio::channel::{Sender, Receiver, channel};
use std::io::{Result, Error, ErrorKind};
use std::collections::BTreeMap;
use event::Event;

const EVENT_LOOP_CHANNEL_TOKEN: Token = Token(0);

pub enum EventHandlerCMD {
    AddEventCallback,
    RemoveEvent,
}

pub struct EventHandlerCommand {
    pub code: EventHandlerCMD,
    pub callback: Vec<Box<Fn(&mut Event)>>,
    pub event_name: Vec<String>,
}

pub struct EventHandler {
    // callbacks fro event handler
    callbacks: BTreeMap<String, Vec<Box<Fn(&mut Event)>>>,

    // channels for communication
    sender_channel: Sender<EventHandlerCommand>,
    receiver_channel: Receiver<EventHandlerCommand>
}


impl EventHandler {
    pub fn new() -> EventHandler {
        let (sender, receiver) = channel::<EventHandlerCommand>();
        EventHandler {
            sender_channel: sender,
            receiver_channel: receiver,
            callbacks: BTreeMap::new()
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // making new poll
        let poll = Poll::new().unwrap();
        match poll.register(&self.receiver_channel, EVENT_LOOP_CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(_) => return Err(Error::new(ErrorKind::Interrupted, "unable to register event loop channel"))
        }

        let mut events: Events = Events::with_capacity(1000);

        loop {
            let event_count = poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.into_iter() {
                if event.token() == EVENT_LOOP_CHANNEL_TOKEN {
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

        Ok(())
    }

    #[inline(always)]
    fn notify(&mut self, cmd: &mut EventHandlerCommand) {
        match cmd.code {
            EventHandlerCMD::AddEventCallback => {
                while !cmd.event_name.is_empty() {
                    let event_name = cmd.event_name.remove(0);
                    let callback = cmd.callback.remove(0);
                    let mut callbacks_vec: Vec<Box<Fn(&mut Event)>> = Vec::new();
                    if self.callbacks.contains_key(&event_name) {
                        callbacks_vec = match self.callbacks.remove(&event_name) {
                            Some(e) => e,
                            None => continue
                        };
                    }

                    callbacks_vec.push(callback);
                    self.callbacks.insert(event_name, callbacks_vec);
                }
            }

            EventHandlerCMD::RemoveEvent => {
                while !cmd.event_name.is_empty() {
                    let event_name = cmd.event_name.remove(0);
                    let _ = self.callbacks.remove(&event_name);
                }
            }
        }
    }

    pub fn on(sender_chan: Sender<EventHandlerCommand>, event_name: String, callback: Box<Fn(&mut Event)>) {
        let _ = sender_chan.send(EventHandlerCommand{
            code: EventHandlerCMD::AddEventCallback,
            event_name: vec![event_name],
            callback: vec![callback]
        });
    }

    pub fn remove(sender_chan: Sender<EventHandlerCommand>, event_name: String) {
        let _ = sender_chan.send(EventHandlerCommand{
            code: EventHandlerCMD::RemoveEvent,
            event_name: vec![event_name],
            callback: vec![]
        });
    }
}
