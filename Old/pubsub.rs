#![allow(dead_code)]

use std::collections::VecDeque;

pub struct EventChannel {
    event_names: Vec<String>
}

// this is the main struct for keeping
// subscribed events, channels, groups and tags
pub struct PubSub {
    messages_queue: VecDeque<Vec<u8>>,

}