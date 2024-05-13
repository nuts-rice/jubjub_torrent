pub mod client;
pub mod config;
pub mod db;
pub mod metrics;
pub mod network;
pub mod parser;
pub mod peer;
pub mod types;

use crate::client::arguments::{get_cmds, Settings};

use libp2p::metrics::{Registry};
use metrics::MetricServer;

use std::error::Error;
use std::sync::{Arc, RwLock};




#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Arc::new(RwLock::new(Settings::new(get_cmds()).await));
    let metrics_registry = Registry::default();
    let metrics = MetricServer::new(metrics_registry, config.clone());
    //moved to network
    let (_network_client, _network_events, network_session) =
        network::new(config, metrics).await.unwrap();
    tokio::spawn(network_session.run());

    Ok(())
}
