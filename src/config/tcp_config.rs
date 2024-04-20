use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub swarm_workers: usize,
    pub network_config: NetworkConfig,
}
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_addresses: Vec<Multiaddr>,
}
