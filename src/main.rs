use libp2p::{identity, PeerId};
use tracing::{info};

pub mod client;
pub mod config;
pub mod db;
pub mod parser;
pub mod peer;
pub mod types;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    client::cli::cli();
    let new_key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(new_key.public());
    info!("Peer id: {:?}", peer_id);
    let mut bytes = vec![2., 13., 8., 1., 2., 3., 4., 5., 6., 7., 8., 9., 10.];
    let mut _bytes = vec![2., 13., 8., 1., 2., 4., 4., 2., 6., 2., 8., 2., 2.];

    parser::chi_squared_test(&_bytes, &bytes);
}
