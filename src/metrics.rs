use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use libp2p::metrics::Registry;
use opentelemetry::KeyValue;
use prometheus_client::encoding::text::encode;
use std::error::Error;

use std::sync::{Arc, RwLock};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

use crate::client::arguments::Settings;
const METRICS_CONTENT_TYPE: &str = "application/openmetrics-text;charset=utf-8;version=1.0.0";
#[derive(Clone)]
pub struct MetricServer {
    registry: Arc<RwLock<Registry>>,
    config: Arc<RwLock<Settings>>,
}

impl MetricServer {
    pub fn new(registry: Arc<RwLock<Registry>>, config: Arc<RwLock<Settings>>) -> Self {
        MetricServer { registry, config }
    }
    fn get_registry(&self) -> Arc<RwLock<Registry>> {
        Arc::clone(&self.registry)
    }
}

pub(crate) fn setup_tracing() -> Result<(), Box<dyn Error>> {
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

pub(crate) async fn metrics_handler(State(server): State<MetricServer>) -> impl IntoResponse {
    let mut buffer = String::new();
    let registry_guard = server.get_registry();
    encode(&mut buffer, &registry_guard.read().unwrap()).unwrap();
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, METRICS_CONTENT_TYPE)],
        buffer,
    )
}

pub(crate) async fn metrics_sink() {
    // loop {
    //     while let
    // }
}

pub(crate) async fn metrics_server(metrics: MetricServer) -> Result<(), std::io::Error> {
    use tokio::net::TcpListener;
    let (addr, _interval, route) = {
        let config_guard = metrics.config.read().unwrap();
        (
            config_guard.metrics.socket_addr,
            config_guard.metrics.update_interval,
            config_guard.metrics.route.clone(),
        )
    };
    let interval = tokio::time::interval(std::time::Duration::from_secs_f64(_interval as f64));
    let metric_server = MetricServer::new(metrics.registry, metrics.config);
    let server = Router::new()
        .route(&route, get(metrics_handler))
        .with_state(metric_server);
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    tracing::info!(metrics_server=%format!("http://{}/metrics", local_addr));
    axum::serve(listener, server.into_make_service()).await?;
    // loop {
    // interval.tick().await;
    // registry.write().unwrap().;
    // Ok(())
    // }
    Ok(())
}
