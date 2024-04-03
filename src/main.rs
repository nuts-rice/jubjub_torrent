use libp2p::{identity, PeerId};
use tracing::{info, debug, error};
use tracing_subscriber::fmt;
pub mod db;
pub mod client;
pub mod peer;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    client::cli::cli();
    let new_key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(new_key.public());
    info!("Peer id: {:?}", peer_id);
}
