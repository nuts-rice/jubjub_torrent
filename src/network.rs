use futures::prelude::*;
use futures::StreamExt;
use libp2p::{
    core::Multiaddr,
    identity, kad,
    request_response::{self, OutboundRequestId, ProtocolSupport, ResponseChannel},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp,
    websocket::tls,
    PeerId, SwarmBuilder,
};
use tracing::info;

use libp2p::StreamProtocol;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map, HashMap, HashSet};
use std::error::Error;
use std::time::Duration;

use crate::client::arguments::Command;
use crate::{
    peer::client::Client,
    types::{TorrentRequest, TorrentResponse},
};

pub(crate) async fn new() -> Result<Client, Box<dyn Error>> {
    let mut identity = identity::Keypair::generate_ed25519();
    let peer_id = identity.public().to_peer_id();
    info!(
        "Peer id: {:?}. Public key: {:?}",
        peer_id,
        identity.public()
    );
    // parser::chi_squared_test(&_bytes, &bytes);
    let mut swarm = SwarmBuilder::with_existing_identity(identity)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|key| Behaviour {
            kademlia: kad::Behaviour::new(
                peer_id,
                kad::store::MemoryStore::new(key.public().to_peer_id()),
            ),
            request_response: request_response::cbor::Behaviour::new(
                [(StreamProtocol::new("/torrent/1"), ProtocolSupport::Full)],
                request_response::Config::default(),
            ),
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(30)))
        .build();
    let (command_tx, command_rx) = tokio::sync::mpsc::channel(10);
    Ok((Client { tx: command_tx }))
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    request_response: request_response::cbor::Behaviour<TorrentRequest, TorrentResponse>,
}
