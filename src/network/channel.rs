#![allow(dead_code)]
pub enum NetworkCMD {
    None,
    ConnectionClose
}

pub struct NetworkCommand {
    pub cmd: NetworkCMD,
    pub token: Vec<String>
}

impl NetworkCommand {
    pub fn new() -> NetworkCommand {
        NetworkCommand {
            cmd: NetworkCMD::None,
            token: vec![]
        }
    }
}