#![allow(dead_code)]

use std::collections::BTreeMap;
use node::{Event, Node};

pub type EventCallback = Box<Fn(&Event, &mut Node) -> bool>;

/// Event callbacks handler for Node
pub struct EventHandler {
    // Events BTreeMap for keeping events and their callbacks
    callbacks: BTreeMap<String, Vec<EventCallback>>,
}

impl EventHandler {
    /// Making new Event Handler with empty event list
    pub fn new() -> EventHandler {
        EventHandler{
            callbacks: BTreeMap::new()
        }
    }

    /// Adding new callback to event
    /// or adding an event with given name if it's not exists
    #[inline(always)]
    pub fn on(&mut self, name: &str, callback: EventCallback) {
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

    /// Removing event from callbacks list
    #[inline(always)]
    pub fn rm(&mut self, name: &str) {
        self.callbacks.remove(&String::from(name));
    }

    /// Run callbacks for specific event name
    /// Using given Event object for callback argument
    #[inline(always)]
    pub fn trigger(&self, event: &Event, node: &mut Node) {
        if !self.callbacks.contains_key(&event.name) {
            return;
        }
        let mut n = node;
        let ref callbacks = self.callbacks[&event.name];
        for cb in callbacks {
            // if callback returning false then breaking the loop
            if !cb(event, &mut n) {
                break;
            }
        }
    }
}