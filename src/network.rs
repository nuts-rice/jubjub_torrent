use futures::prelude::*;
use futures::StreamExt;
use hashbrown::HashMap;
use libp2p::Multiaddr;

use crate::client::arguments::ClientCommand;
use crate::client::arguments::Settings;
use crate::metrics::MetricServer;
use crate::peer::client::ClientMode;
use crate::types;
use crate::types::Event;
use crate::{
    peer::client::Client,
    types::{TorrentRequest, TorrentResponse},
};
use futures::channel::{mpsc, oneshot};
use libp2p::StreamProtocol;
use libp2p::{
    identity, kad,
    multiaddr::Protocol,
    request_response::{self, OutboundRequestId, ProtocolSupport},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, PeerId, SwarmBuilder,
};
use prometheus_client::registry::Registry;
use std::error::Error;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use thiserror::Error;
use tracing::info;
const BOOTNODES: [&str; 4] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
];
const IPFS_PROTO_NAME: StreamProtocol = StreamProtocol::new("/ipfs/kad/1.0.0");

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Failed to send Request command to client: {0}")]
    RequestResponseError(String),
    #[error("Provider error for {0}: {1}")]
    ProviderError(String, String),
    #[error("Failed to set up new network connection: {0} ")]
    CreateError(String),
    #[error("Failed to create swarm for identity: {0}")]
    SwarmError(String),
}

pub(crate) async fn new(
    config: Arc<RwLock<Settings>>,
    metrics: MetricServer,
    mode: ClientMode,
    secret_key: Option<u8>,
) -> Result<(Client, impl Stream<Item = Event>, Session), Box<dyn Error>> {
    let keys = match secret_key {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            identity::Keypair::ed25519_from_bytes(bytes).unwrap()
        }
        None => identity::Keypair::generate_ed25519(),
    };
    let peer_id = keys.public().to_peer_id();
    let mut metric_registry = Registry::default();
    let (
        //ipfs_path, ipfs_addr, ipfs_workers,
        tcp_addr,
        workers,
        download_dir,
    ) = {
        let config_guard = config.read().unwrap();
        (
            config_guard.tcp.address.clone(),
            config_guard.tcp.socket_workers,
            config_guard.download_dir.clone(),
        )
    };
    info!("Peer id: {:?}. Public key: {:?}", peer_id, keys.public());
    // parser::chi_squared_test(&_bytes, &bytes);
    let mut swarm = SwarmBuilder::with_existing_identity(keys)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        // .unwrap()
        .with_bandwidth_metrics(&mut metric_registry)
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
    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));
    // let address: Multiaddr = (format!("/ip4/{}/tcp/{}", tcp_addr.ip(), tcp_addr.port()))
    //     .parse()
    //     .unwrap();
    // swarm.listen_on(address)?;
    let (command_tx, command_rx) = mpsc::channel(32);
    let (event_tx, event_rx) = mpsc::channel(32);
    Ok((
        Client {
            tx: command_tx,
            mode,
        },
        event_rx,
        Session::new(swarm, metrics, command_rx, event_tx),
    ))
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    request_response: request_response::cbor::Behaviour<TorrentRequest, TorrentResponse>,
}

pub(crate) struct Session {
    swarm: Swarm<Behaviour>,
    metrics: MetricServer,
    command_rx: mpsc::Receiver<ClientCommand>,
    event_tx: mpsc::Sender<types::Event>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    request_cmd_map: HashMap<OutboundRequestId, ClientCommand>,
    provider_query_tx_map: HashMap<kad::QueryId, oneshot::Sender<()>>,
    request_file_map:
        HashMap<OutboundRequestId, oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>>,
    query_peer_map: HashMap<kad::QueryId, oneshot::Sender<std::collections::HashSet<PeerId>>>,
}

impl Session {
    pub fn new(
        swarm: Swarm<Behaviour>,
        metrics: MetricServer,
        command_rx: mpsc::Receiver<ClientCommand>,
        event_tx: mpsc::Sender<types::Event>,
    ) -> Self {
        Self {
            swarm,
            metrics,
            command_rx,
            event_tx,
            pending_dial: Default::default(),
            request_cmd_map: Default::default(),
            provider_query_tx_map: Default::default(),
            request_file_map: Default::default(),
            query_peer_map: Default::default(),
        }
    }

    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                command = self.command_rx.next() => match command {
                    Some(command) => self.handle_command(command).await,
                    None => return,
                }
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    id,
                    result: kad::QueryResult::StartProviding(_),
                    ..
                },
            )) => {
                let sender: oneshot::Sender<()> = self
                    .provider_query_tx_map
                    .remove(&id)
                    .expect("missing provider query tx");
                let _ = sender.send(());
            }
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    id,
                    result:
                        kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
                            providers,
                            ..
                        })),
                    ..
                },
            )) => {
                if let Some(sender) = self.query_peer_map.remove(&id) {
                    sender.send(providers).expect("Reciever dropped");
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .query_mut(&id)
                        .unwrap()
                        .finish();
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    result:
                        kad::QueryResult::GetProviders(Ok(
                            kad::GetProvidersOk::FinishedWithNoAdditionalRecord { .. },
                        )),
                    ..
                },
            )) => {}
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(_)) => {}
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    self.event_tx
                        .send(Event::InboundRequest {
                            request: request.0,
                            channel,
                        })
                        .await
                        .expect("event tx dropped");
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    let _ = self
                        .request_file_map
                        .remove(&request_id)
                        .expect("request still pending")
                        .send(Ok(response.0));
                }
            },
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure {
                    request_id, error, ..
                },
            )) => {
                let _ = self
                    .request_file_map
                    .remove(&request_id)
                    .expect("request still pending")
                    .send(Err(Box::new(error)));
            }
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::ResponseSent { .. },
            )) => {}
            SwarmEvent::NewListenAddr { address, .. } => {
                let local_peer_id = *self.swarm.local_peer_id();
                info!(
                    "Listening on {:?}",
                    address.with(Protocol::P2p(local_peer_id))
                );
            }
            SwarmEvent::IncomingConnection { .. } => {}
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    info!("Dialer {:?} connected at {:?}", peer_id, endpoint);
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::ConnectionClosed { .. } => {}
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(Box::new(error)));
                    }
                }
            }
            SwarmEvent::IncomingConnectionError { .. } => {}
            SwarmEvent::Dialing {
                peer_id: Some(peer_id),
                ..
            } => info!("Dialing {:?}", peer_id),
            e => panic!("Unhandled swarm event {:?}", e),
        }
    }

    async fn handle_command(&mut self, command: ClientCommand) {
        match command {
            ClientCommand::DecodeCommand { val: _ } => {
                unimplemented!()
            }
            ClientCommand::TorrentInfoCommand { torrent: _ } => {
                unimplemented!()
            }
            ClientCommand::GetFileCommand {
                output: _,
                torrent,
                peer,
                tx,
            } => {
                let request_id = self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, TorrentRequest(torrent));
                self.request_file_map.insert(request_id, tx);
            }
            ClientCommand::GetPeersCommand { torrent, tx } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_providers(torrent.into_bytes().into());
                self.query_peer_map.insert(query_id, tx);
            }
            ClientCommand::DialCommand {
                peer_id,
                torrent: _,
                addr,
                tx,
            } => {
                if let hashbrown::hash_map::Entry::Vacant(e) = self.pending_dial.entry(peer_id) {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr.clone());
                    match self.swarm.dial(addr.with(Protocol::P2p(peer_id))) {
                        Ok(()) => {
                            e.insert(tx);
                        }
                        Err(e) => {
                            let _ = tx.send(Err(Box::new(e)));
                        }
                    }
                } else {
                    todo!("Handle dial command")
                }
            }
            ClientCommand::ListenCommand { addr, tx } => {
                let _ = match self.swarm.listen_on(addr) {
                    Ok(_) => tx.send(Ok(())),
                    Err(e) => tx.send(Err(Box::new(e))),
                };
            }
            ClientCommand::ProvideTorrent {
                file: _,
                channel: _,
            } => {
                unimplemented!()
            }
        }
    }

    // unimplemented!()
    // match command {}
}
async fn fetch_providers() -> Result<ProviderResult, NetworkError> {
    todo!()
}

//override provider result
#[derive(Debug, Clone)]
pub enum ProviderResult {
    NewProviders {
        key: libp2p::kad::RecordKey,
        providers: hashbrown::HashSet<PeerId>,
    },
    NoNewProviders {
        closest_peers: Vec<PeerId>,
    },
}
//TODO: collect downloaded pieces, spawn this as a thread , use a channel
async fn handle_provider() {}

//TODO: serves pieces to download, spawn this as a thread , use a channel
async fn serve_request() {}

pub(crate) async fn metrics_server(_registry: Registry) -> Result<(), std::io::Error> {
    unimplemented!()
}
#[cfg(test)]
mod tests {
    use libp2p::Multiaddr;
    use std::sync::Arc;

    async fn create_mock_config() -> Arc<RwLock<Settings>> {
        let config = Settings::new(get_cmds()).await;
        Arc::new(RwLock::new(config))
    }
    use super::*;
    use crate::get_cmds;
    #[tokio::test]
    //RUST_LOG=info cargo test  -- test_network --nocapture
    async fn test_network() {
        let config_rwlock = create_mock_config().await;
        let registry_rwlock = Arc::new(RwLock::new(Registry::default()));
        let metrics = MetricServer::new(registry_rwlock, config_rwlock.clone());
        let (mut network_client, network_events, network_session) = new(
            config_rwlock.clone(),
            metrics,
            ClientMode::Download,
            Some(0u8),
        )
        .await
        .unwrap();
        let address = config_rwlock.read().unwrap().tcp.address.clone();
        let (tx, rx) = oneshot::channel();
        tokio::spawn(network_session.run());
        let command = ClientCommand::ListenCommand { addr: address, tx };
        network_client.tx.send(command).await.unwrap();
        let result = rx.await.unwrap();
        info!("{:?}", result);
    }
}
