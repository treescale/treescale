#![allow(dead_code)]

use network::ConnectionIdentity;

pub enum NetworkCMD {
    None,
    ConnectionClose,
    HandleConnection
}

pub struct NetworkCommand {
    pub cmd: NetworkCMD,
    pub token: Vec<String>,
    pub value: Vec<u64>,
    pub conn_identity: Vec<ConnectionIdentity>
}

impl NetworkCommand {
    pub fn new() -> NetworkCommand {
        NetworkCommand {
            cmd: NetworkCMD::None,
            token: vec![],
            value: vec![],
            conn_identity: vec![]
        }
    }
}