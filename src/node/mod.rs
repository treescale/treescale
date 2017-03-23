#![allow(dead_code)]

extern crate mio;
mod main;

pub use self::main::Node;


use self::mio::Token;
use std::u32::MAX as u32MAX;

pub const NET_RECEIVER_CHANNEL_TOKEN: Token = Token((u32MAX - 1) as usize);
pub const NET_TCP_SERVER_TOKEN: Token = Token((u32MAX - 2) as usize);

pub const EVENT_LOOP_EVENTS_SIZE: usize = 65000;
pub const DEFAULT_API_VERSION: u32 = 1;