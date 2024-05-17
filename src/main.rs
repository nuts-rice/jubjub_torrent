pub mod client;
pub mod config;
pub mod db;
pub mod metrics;
pub mod network;
pub mod parser;
pub mod peer;
pub mod types;

use crate::client::arguments::{get_cmds, Settings};
use axum::Router;
use client::arguments::ClientCommand;
use libp2p::metrics::Registry;
use metrics::{setup_tracing, MetricServer};

use std::error::Error;
use std::sync::{Arc, RwLock};

pub struct App {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_rwlock = Arc::new(RwLock::new(Settings::new(get_cmds()).await));
    let metrics_registry = Registry::default();
    let registry_rwlock = Arc::new(RwLock::new(metrics_registry));
    let metrics = MetricServer::new(registry_rwlock.clone(), config_rwlock.clone());
    //moved to network
    let (_network_client, _network_events, network_session) =
        network::new(config_rwlock.clone(), metrics.clone())
            .await
            .unwrap();
    tokio::spawn(network_session.run());
    setup_tracing();
    tokio::spawn(metrics::metrics_server(metrics));
    Ok(())
}
