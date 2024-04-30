use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct WSConfig {
    pub socket_workers: usize,
    network: NetworkConfig,
}

impl Default for WSConfig {
    fn default() -> Self {
        Self {
            socket_workers: 4,
            network: NetworkConfig {},
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct NetworkConfig {}
