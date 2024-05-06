use futures::prelude::*;
use futures::StreamExt;
use hashbrown::HashMap;

use libp2p::metrics::Registry;
use libp2p::StreamProtocol;
use libp2p::{
    identity, kad,
    multiaddr::Protocol,
    request_response::{self, OutboundRequestId, ProtocolSupport},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, PeerId, SwarmBuilder,
};
use tracing::info;

use crate::client::arguments::ClientCommand;
use crate::types;
use crate::types::Event;
use crate::{
    peer::client::Client,
    types::{TorrentRequest, TorrentResponse},
};
use futures::channel::{mpsc, oneshot};

use std::error::Error;
use std::time::Duration;

pub(crate) async fn new() -> Result<(Client, impl Stream<Item = Event>, Session), Box<dyn Error>> {
    let identity = identity::Keypair::generate_ed25519();
    let peer_id = identity.public().to_peer_id();
    info!(
        "Peer id: {:?}. Public key: {:?}",
        peer_id,
        identity.public()
    );
    // parser::chi_squared_test(&_bytes, &bytes);
    let swarm = SwarmBuilder::with_existing_identity(identity)
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
    let (command_tx, command_rx) = mpsc::channel(0);
    let (event_tx, event_rx) = mpsc::channel(0);

    Ok((
        Client { tx: command_tx },
        event_rx,
        Session::new(swarm, command_rx, event_tx),
    ))
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    request_response: request_response::cbor::Behaviour<TorrentRequest, TorrentResponse>,
}

pub(crate) struct Session {
    swarm: Swarm<Behaviour>,
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
    fn new(
        swarm: Swarm<Behaviour>,
        command_rx: mpsc::Receiver<ClientCommand>,
        event_tx: mpsc::Sender<types::Event>,
    ) -> Self {
        Self {
            swarm,
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
                Some(command) = self.command_rx.next() => self.handle_command(command).await,
                event = self.swarm.select_next_some() => self.handle_event(event).await,
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
pub fn fetch_providers() {}

pub type FetchProvidersResult = Result<ProviderResult, ProviderError>;
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
#[derive(Debug, Clone)]
pub enum ProviderError {}

fn provider_result() {}

pub(crate) async fn metrics_server(_registry: Registry) -> Result<(), std::io::Error> {
    unimplemented!()
}
#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_network() {
        let (mut network_client, mut network_events, network_session) = new().await.unwrap();
        tokio::spawn(network_session.run());
        // let command = Command::ListenCommand {
        //     addr: "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
        // };
    }
}
