use crate::client::arguments::Command;
use async_trait::async_trait;
// use url::{Url, ParseError};
use libp2p::PeerId;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::sync::mpsc::{Receiver, Sender};

pub trait Node {
    fn get_peer_id(&self) -> u32;
}

pub enum Event {}

pub struct Session {
    // swarm: Swarm<>,
    cmd_rx: Receiver<Command>,
    event_tx: Sender<Event>,
}

pub struct Torrent {
    announce: Option<String>,
    pub info_hash: Vec<u8>,
}

impl Torrent {
    pub fn new(announce: Option<String>, info_hash: Vec<u8>) -> Self {
        Self {
            announce,
            info_hash,
        }
    }

    async fn open(file: &str) -> Result<Self, Box<dyn Error>> {
        let torrent = std::fs::read(file).unwrap();

        let torrent = Torrent::from_bytes(&torrent).unwrap();
        Ok(torrent)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let announce = serde_bencode::from_bytes::<String>(bytes)?;
        let info_hash = serde_bencode::from_bytes::<Vec<u8>>(bytes)?;
        let torrent = Torrent::new(Some(announce), info_hash);
        Ok(torrent)
    }
}

#[derive(Deserialize, Serialize)]
pub struct RequestHeader {}

#[async_trait]
pub trait TorrentFile {
    type File;
    async fn from_file(file: &str) -> Self;
    async fn from_bytes(bytes: &[u8]) -> Self;
    async fn build_tracker_URL(&self) -> anyhow::Result<String>;
}

#[derive(Deserialize, Serialize)]
pub struct File {
    pub header: RequestHeader,
    pub announce: Option<String>,
    pub info_hash: Vec<u8>,
    pub piece_hash: Vec<u8>,
    pub piece_length: u64,
    pub length: usize,
    pub name: String,
}

impl File {
    pub fn new(
        announce: Option<String>,
        info_hash: Vec<u8>,
        piece_hash: Vec<u8>,
        piece_length: u64,
        length: usize,
        name: String,
        request_header: RequestHeader,
    ) -> Self {
        Self {
            header: request_header,
            announce,
            info_hash,
            piece_hash,
            piece_length,
            length,
            name,
        }
    }

    async fn from_bytes(_bytes: &[u8]) -> Self {
        unimplemented!()
    }
    async fn build_tracker_URL(&self, peer_id: PeerId, port: u16) -> Result<Url, Box<dyn Error>> {
        let mut url = Url::parse(self.announce.as_ref().unwrap().as_str())?;
        // let params = [
        //     ("info_hash", self.info_hash),
        //     ("peer_id", peer_id.to_bytes()),
        //     ("port", port.to_string().into_bytes()),
        //     ("uploaded", 0.to_string().into_bytes()),
        //     ("downloaded", 0.to_string().into_bytes()),
        //     ("left", self.length.to_string().into_bytes()),
        // ];
        // url.query_pairs_mut().append_pair("info_hash", &self.info_hash);
        url.query_pairs_mut()
            .append_pair("peer_id", peer_id.to_string().as_str())
            .append_pair("port", port.to_string().as_str())
            .append_pair("uploaded", 0.to_string().as_str())
            .append_pair("downloaded", 0.to_string().as_str())
            .append_pair("left", self.length.to_string().as_str());
        Ok(url)
    }

    async fn check_piece(&self, _index: u64) -> Result<bool, Box<dyn Error>> {
        unimplemented!()
    }
}
mod tests {
use super::*;    
use tracing::info;
    
    

    #[tokio::test]
    async fn test_tracker_URL() {
        let peer_id = PeerId::random();
        let file = File::new(
            Some("http://tracker.com".to_string()),
            vec![0; 20],
            vec![0; 20],
            10,
            100,
            "test".to_string(),
            RequestHeader {},
        );
        let url = file.build_tracker_URL(peer_id, 9091).await.unwrap();
        info!("url: {}", url);
        let expected = format!(
            "http://tracker.com/?peer_id={}&port=9091&uploaded=0&downloaded=0&left=100",
            peer_id
        );
        assert_eq!(
            file.build_tracker_URL(peer_id, 9091)
                .await
                .unwrap()
                .to_string(),
            expected.to_string(),
        );
    }
}
