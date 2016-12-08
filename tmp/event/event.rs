#![allow(dead_code)]
#![feature(proc_macro)]
extern crate serde;
extern crate rmp_serde;

use std::io::{Result};
use std::sync::Arc;
use self::serde::Deserialize;

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub name: String,
    pub from: String,
    pub target: String,
    pub data: String,
}

impl Event {
    pub fn from_raw(data: Arc<Vec<u8>>) -> Result<Event> {
        msgpack::from_msgpack(data.as_slice());
    }
}
