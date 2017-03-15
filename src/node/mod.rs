#![allow(dead_code)]

extern crate mio;
mod main;

pub use self::main::Node;


use self::mio::Token;
use std::u32::MAX as u32MAX;

pub const NET_RECEIVER_CHANNEL_TOKEN: Token = Token((u32MAX - 1) as usize);