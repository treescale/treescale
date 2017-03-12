#![allow(dead_code)]
use network::NetworkConfig;

pub struct NodeConfig {
    pub token: String,
    pub value: u64,
    pub network: NetworkConfig
}

impl NodeConfig {
    /// Making default configurations
    pub fn default() -> NodeConfig {
        NodeConfig {
            token: String::new(),
            value: 0,
            network: NetworkConfig::default()
        }
    }
}