use crate::client::arguments::ClientCommand;
use crate::peer::error::ClientError;
use crate::types::Node;
use crate::types::Torrent;
use ::futures::SinkExt;
use libp2p::futures::channel::{mpsc, oneshot};
use libp2p::Multiaddr;
use libp2p::PeerId;
use serde_bencode as bencode;
use serde_json as json;

use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;
#[derive(Clone)]
pub struct Client {
    pub tx: mpsc::Sender<ClientCommand>,
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
    // pub async fn new(seed: Option<u8>) -> Result<(Client, impl Stream<Item = Event> ,Session), Box<dyn Error>> {
    //     unimplemented!()
    // }
    pub(crate) async fn start_listening(
        &mut self,
        addr: Multiaddr,
    ) -> Result<json::Value, ClientError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ClientCommand::ListenCommand { addr, tx })
            .await
            .expect("Receiver not dropped yet...");
        rx.await.expect("Sender not dropped yet...");
        let res = json::json!({
            "result": "Listening on {:?}",
        });
        Ok(res)
    }
    pub async fn get_file(&self, file: &str) -> Result<json::Value, ClientError> {
        let file = Torrent::open(file);

        let id = self.get_peer_id();
        unimplemented!()
    }

    pub(crate) async fn execute_command(
        mut self,
        tx: serde_json::Value,
    ) -> Result<json::Value, ClientError> {
        let method = tx["method"].as_str();
        println!("Method: {:?}", method.unwrap_or("None"));
        match method {
            Some("provide") => {
                let file = tx["params"]["file"].as_str().unwrap();
                Client::start_providing(&mut self, file.to_string()).await
            }
            Some("get") => {
                let file = tx["params"]["file"].as_str().unwrap();
                Client::get_file(&self, file).await
            }
            Some("listen") => {
                let addr = tx["params"]["addr"].as_str().unwrap();
                Client::start_listening(&mut self, addr.parse().unwrap()).await
            }
            Some("dial") => {
                let addr = tx["params"]["addr"].as_str().unwrap();
                let peer_id = tx["params"]["peer_id"].as_str().unwrap();
                let torrent = tx["params"]["torrent"].as_str().unwrap();
                Client::dial(
                    &mut self,
                    addr.parse().unwrap(),
                    PeerId::from_str(peer_id).unwrap(),
                    torrent,
                )
                .await
            }
            Some("get_peers") => {
                let file = tx["params"]["file"].as_str().unwrap();
                Client::get_peers(&mut self, file.to_string()).await
            }
            Some(_) => Err(ClientError::InvalidMethod),
            _ => Ok(().into()),
        }
    }

    pub(crate) async fn start_providing(
        &mut self,
        file: String,
    ) -> Result<json::Value, ClientError> {
        let (tx, _rx) = oneshot::channel();
        let bytes = std::fs::read(file.clone()).expect("File not found");
        self.tx
            .send(ClientCommand::ProvideTorrent {
                file: bytes,
                channel: tx,
            })
            .await
            .expect("Receiver not dropped yet...");
        let res = json::json!({
        "result": "Providing {:?}",
                    });
        Ok(res)
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
    ) -> Result<json::Value, ClientError> {
        let (_tx, rx) = oneshot::channel();
        self.tx
            .send(ClientCommand::DialCommand {
                peer_id,
                torrent: PathBuf::from(torrent),
                addr,
                tx: _tx,
            })
            .await
            .expect("Receiver not dropped yet...");
        rx.await.expect("Sender not dropped yet...");
        let res = json::json!({
            "result": "Dialing {:?}",
        });
        Ok(res)
    }

    async fn get_peers(&mut self, file: String) -> Result<json::Value, ClientError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ClientCommand::GetPeersCommand { tx, torrent: file })
            .await
            .expect("Receiver not dropped yet...");
        rx.await.expect("Sender not dropped yet...");
        let res = json::json!({
            "result": "Getting peers for {:?}",
        });
        Ok(res)
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
