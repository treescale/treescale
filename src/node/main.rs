#![allow(dead_code)]
use event::EventCallback;
use std::collections::BTreeMap;

pub struct Node {
    pub callbacks: BTreeMap<String, Vec<EventCallback>>
}