use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct WSConfig {
    pub socket_workers: usize,
}

impl Default for WSConfig {
    fn default() -> Self {
        Self { socket_workers: 4 }
    }
}
