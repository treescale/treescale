#![allow(dead_code)]
use node::Node;
use event::Event;

pub type EventCallback = Box<Fn(&Event, &mut Node) -> bool>;

pub trait EventHandler {
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
}

impl EventHandler for Node {
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
}

pub enum EventCMD {
    None
}

pub struct EventCommand {
    cmd: EventCMD
}

impl EventCommand {
    #[inline(always)]
    pub fn new() -> EventCommand {
        EventCommand {
            cmd: EventCMD::None
        }
    }
}
