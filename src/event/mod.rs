#![allow(dead_code)]
mod event;
mod handler;

pub use self::event::Event;
pub use self::handler::{EventHandler, EventHandlerCommand, EventHandlerCMD};

// Defining default events
pub const EVENT_ON_PENDING_CONNECTION: &'static str = "__!!__on_pending_connection";
