pub mod client;
pub mod config;
pub mod db;
pub mod metrics;
pub mod network;
pub mod parser;
pub mod peer;
pub mod types;

use crate::client::arguments::{get_cmds, Settings};
use crate::metrics::{metrics_server, setup_tracing};
use libp2p::metrics::{Metrics, Registry};
use opentelemetry::KeyValue;
use std::error::Error;
use std::sync::{Arc, RwLock};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Arc::new(RwLock::new(Settings::new(get_cmds()).await));
    setup_tracing().unwrap();
    let mut metrics_registry = Registry::default();
    //moved to network
    let (mut network_client, mut network_events, network_session) = network::new().await.unwrap();
    tokio::spawn(network_session.run());

    let _ = setup_tracing();
    let _metrics = Metrics::new(&mut metrics_registry);
    Ok(())
}
