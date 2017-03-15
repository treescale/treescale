#![allow(dead_code)]
pub enum NetworkCMD {
    None
}

pub struct NetworkCommand {
    pub cmd: NetworkCMD
}

impl NetworkCommand {
    pub fn new() -> NetworkCommand {
        NetworkCommand {
            cmd: NetworkCMD::None
        }
    }
}