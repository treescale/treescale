#![allow(dead_code)]
pub struct NodeConfig {
    pub value: u64,
    pub token: String,
    pub api_version: u32,
    pub network: NetworkingConfig
}

pub struct NetworkingConfig {
    pub tcp_server_host: String,
    pub concurrency: usize
}