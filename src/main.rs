pub mod client;
pub mod config;
pub mod db;
pub mod network;
pub mod parser;
pub mod peer;
pub mod types;


use libp2p::metrics::{Metrics, Registry};
use opentelemetry::KeyValue;
use std::error::Error;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
#[tokio::main]
async fn main() {
    client::cli::cli();
    setup_tracing().unwrap();
    let mut metrics_registry = Registry::default();
    //moved to network
    let (_network_client, _network_events, network_session) = network::new().await.unwrap();
    tokio::spawn(network_session.run());
    // let command = crate::client::arguments::Command::ListenCommand {
    //     addr: "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
    //     tx,

    // };
    let _metrics = Metrics::new(&mut metrics_registry);
    // tokio::spawn(network::metrics_server(metrics_registry));
    // loop {
    //     match
}

fn setup_tracing() -> Result<(), Box<dyn Error>> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
            opentelemetry_sdk::Resource::new(vec![KeyValue::new("service.torrent", "libp2p")]),
        ))
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env()))
        .with(
            tracing_opentelemetry::layer()
                .with_tracer(tracer)
                .with_filter(EnvFilter::from_default_env()),
        )
        .try_init()?;
    Ok(())
}
