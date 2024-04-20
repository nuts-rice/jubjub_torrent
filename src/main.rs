use libp2p::{
    identity, kad,
    request_response::{self, OutboundRequestId, ProtocolSupport, ResponseChannel},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp,
    websocket::tls,
    PeerId, SwarmBuilder,
};
use tracing::info;

pub mod client;
pub mod config;
pub mod db;
pub mod network;
pub mod parser;
pub mod peer;
pub mod types;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    client::cli::cli();
    let bytes = vec![2., 13., 8., 1., 2., 3., 4., 5., 6., 7., 8., 9., 10.];
    let mut _bytes = vec![2., 13., 8., 1., 2., 4., 4., 2., 6., 2., 8., 2., 2.];
    //moved to network
    let (mut network) = network::new().await;
    unimplemented!()
}
