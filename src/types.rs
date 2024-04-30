use async_trait::async_trait;
// use url::{Url, ParseError};
use libp2p::{request_response::ResponseChannel, PeerId};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, error::Error};

use crate::client::arguments::Command;

pub trait Node {
    fn get_peer_id(&self) -> u32;
}
#[derive(Debug)]
pub(crate) enum Event {
    InboundRequest {
        request: String,
        channel: ResponseChannel<TorrentResponse>,
    },
}

pub type SessionId = u32;

pub(crate) struct Torrent {
    announce: Option<String>,
    pub info_hash: InfoHash,
    peers: hashbrown::HashMap<PeerId, SessionId>,
    cmd_rx: futures::channel::mpsc::Receiver<Command>,
    listen_addr: libp2p::Multiaddr,
    started: Option<std::time::Instant>,
    completed: Option<Vec<usize>>,
}

impl Torrent {
    pub fn new(announce: Option<String>, info_hash: Vec<u8>) -> Self {
        let (_cmd_tx, cmd_rx) = futures::channel::mpsc::channel(10);
        let listen_addr = "/ip4/".parse().unwrap();
        Self {
            announce,
            info_hash: InfoHash {
                hash: info_hash.try_into().unwrap(),
            },
            peers: hashbrown::HashMap::new(),
            cmd_rx,
            listen_addr,
            started: None,
            completed: None,
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
        // let mut opt_info_hash = None;
        // let mut opt_peer_id = None;
        // let mut opt_port = None;
        // let mut opt_bytes_left = None;

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

    // async fn request_peers(&self, peer_id: PeerId, port: u16, peer_addr: Multiaddr) -> Result<(), Box<dyn Error>> {
    //     use tokio::sync::oneshot;
    //     let url = self.build_tracker_URL(peer_id, port).await?;
    //     let (tx, rx) = oneshot::channel();

    //     let client = reqwest::Client::new();

    //     unimplemented!()
    // }

    async fn check_piece(&self, _index: u64) -> Result<bool, Box<dyn Error>> {
        unimplemented!()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct TorrentRequest(pub String);
#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct TorrentResponse(pub Vec<u8>);

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct PieceRequest(pub String);
#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct PieceResponse(pub Vec<HashSet<String>>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListRequest {
    pub info_hashes: Vec<[u8; 20]>,
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Clone)]
pub struct InfoHash {
    hash: [u8; 20],
}

impl std::fmt::Display for InfoHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rx: [u8; 20] = serde_bencode::from_bytes(&self.hash).expect("failed to serialize");
        write!(f, "{:?}", std::str::from_utf8(&rx).unwrap())
    }
}

#[derive(Debug)]
pub enum ChannelRequest {}

mod tests {
    
    

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
