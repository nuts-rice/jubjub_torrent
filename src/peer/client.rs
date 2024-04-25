use crate::client::arguments::Command;
use crate::types::Node;
use crate::types::Torrent;
use libp2p::futures::channel::{mpsc, oneshot};
use ::futures::SinkExt;
use libp2p::Multiaddr;
use libp2p::PeerId;
use serde_bencode as bencode;
use serde_json as json;
use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;
#[derive(Clone)]
pub struct Client {
    pub tx: mpsc::Sender<Command>,
}

#[repr(C)]
pub struct Handshake {
    pub len: u8,
    pub bittorent: [u8; 19],
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Handshake {
            len: 19,
            bittorent: *b"BitTorrent protocol",
            reserved: [0; 8],
            info_hash,
            peer_id,
        }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unimplemented!()
    }
}

impl Node for Client {
    fn get_peer_id(&self) -> u32 {
        unimplemented!()
    }
}

impl Client {
    async fn handle_command(&mut self, command: Command) -> Result<(), Box<dyn Error>> {
        match command {
            Command::DecodeCommand { val: _ } => {
                unimplemented!()
            }
            Command::TorrentInfoCommand { torrent: _ } => {
                unimplemented!()
            }
            Command::GetFileCommand {
                output: _,
                torrent: _,
                peer: _,
            } => {
                unimplemented!()
            }
            Command::GetPeersCommand { torrent: _ , .. } => {
                unimplemented!()
            }
            Command::DialCommand {
                peer_id: _,
                torrent: _,
                addr: _,
                ..
            } => {
                unimplemented!()
            }
            Command::ListenCommand { addr: _ } => {
                unimplemented!()
            }
            Command::ProvideTorrent { file, channel } => {
                unimplemented!()
            }
                    
        }
    }
    // pub async fn new(seed: Option<u8>) -> Result<(Client, impl Stream<Item = Event> ,Session), Box<dyn Error>> {
    //     unimplemented!()
    // }
    pub async fn get_file(&self, _file: String) -> Result<(), Box<dyn Error>> {
        let _id = self.get_peer_id();
        Ok(())
    }

    pub fn decode_value(val: String) -> (json::Value, String) {
        let serialized = bencode::to_string(&val).unwrap();
        let res = json::Value::String(val.to_string());
        (res, serialized)
    }

    pub fn decode_file(_file: PathBuf) -> (Torrent, String) {
        unimplemented!()
        // let file = std::fs::read_to_string(file)?;
        // let file = bencode::from_str(&file)?;
        // Ok(file)
    }

    async fn dial(
        &mut self,
        addr: Multiaddr,
        peer_id: PeerId,
        torrent: &str,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (_tx, rx) = oneshot::channel();
        self.tx
            .send(Command::DialCommand {
                peer_id,
                torrent: PathBuf::from(torrent),
                addr,
                tx: _tx,
            })
            .await
            .expect("Receiver not dropped yet...");
        rx.await.expect("Sender not dropped yet...")
    }

    async fn get_peers(&mut self, file: String) -> hashbrown::HashSet<PeerId> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Command::GetPeersCommand {
                tx,
                torrent: PathBuf::from(file),
            })
            .await
            .expect("Receiver not dropped yet...");
        rx.await.expect("Sender not dropped yet...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;
    use tracing_test::traced_test;
    #[test]
    #[traced_test]
    fn test_decode_value() {
        let (val, serialized) = Client::decode_value("d4:spam3:egge".to_string());
        info!(
            "decoded bencode val: {:?}, serialized: {:?}",
            val, serialized
        );
    }
}
