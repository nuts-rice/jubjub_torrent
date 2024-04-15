use crate::client::arguments::Command;
use tokio::sync::mpsc::{Sender, Receiver};
use libp2p::{core::Multiaddr, identity, kad, multiaddr::Protocol, noise, swarm::{NetworkBehaviour, Swarm, SwarmEvent}, tcp, yamux, PeerId};
pub trait Node{
    fn get_peer_id(&self) -> u32;
}


pub enum Event{}

pub struct Session{
    // swarm: Swarm<>,
    cmd_rx: Receiver<Command>,
    event_tx: Sender<Event>,


}

