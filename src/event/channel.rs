#![allow(dead_code)]

use event::Event;

pub enum EventCMD {
    None,
    HandleEvent
}

pub struct EventCommand {
    pub cmd: EventCMD,
    pub token: Vec<String>,
    pub event: Vec<Event>
}

impl EventCommand {
    #[inline(always)]
    pub fn new() -> EventCommand {
        EventCommand {
            cmd: EventCMD::None,
            token: vec![],
            event: vec![]
        }
    }
}
