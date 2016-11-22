#![allow(dead_code)]
#![allow(unreachable_code)]
extern crate mio;

use self::mio::{Poll, Token, Ready, PollOpt, Events};
use self::mio::channel::{Sender, Receiver, channel};
use std::io::{Result, Error, ErrorKind};
use event::Event;

const EVENT_LOOP_CHANNEL_TOKEN: Token = Token(0);

struct EventHandler {
    sender_channel: Sender<Box<Fn(&mut Event)>>,
    receiver_channel: Receiver<Box<Fn(&mut Event)>>
}


impl EventHandler {
    pub fn new() -> EventHandler {
        let (sender, receiver) = channel::<Box<Fn(&mut Event)>>();
        EventHandler {
            sender_channel: sender,
            receiver_channel: receiver
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

            }
        }

        Ok(())
    }
}
