use cratetorrent::prelude::*;
// use url::{Url, ParseError};
use libp2p::{request_response::ResponseChannel, PeerId};

use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, error::Error, path::PathBuf};
use strum::Display;

use crate::client::arguments::ClientCommand;

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

pub type PeerMap = hashbrown::HashMap<PeerId, SessionId>;

#[derive(Debug)]
pub struct Torrent {
    announce: Option<String>,
    pub info_hash: InfoHash,
    peers: PeerMap,
    cmd_rx: futures::channel::mpsc::Receiver<ClientCommand>,
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

    pub(crate) async fn open(file: &str) -> Result<Self, Box<dyn Error>> {
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

fn decode_torrent(torrent: &Torrent) {}

#[derive(Deserialize, Serialize)]
pub struct RequestHeader {
    request: Request,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Paused = 0,
    Downloading = 1,
    DownloadQueued = 2,
    Seeding = 3,
    SeedQueued = 4,
}

#[derive(Deserialize, Serialize)]
pub struct File {
    pub header: RequestHeader,
    pub id: String,
    pub seeders: u32,
    pub leechers: u32,
    pub announce: Option<String>,
    pub info_hash: InfoHash,
    pub piece_hash: Vec<u8>,
    pub piece_length: u64,
    pub length: usize,
    pub name: String,
    pub destination: Option<PathBuf>,
}

impl File {
    pub fn new(
        announce: Option<String>,
        id: String,
        seeders: u32,
        leechers: u32,
        info_hash: InfoHash,
        piece_hash: Vec<u8>,
        piece_length: u64,
        length: usize,
        name: String,
        header: RequestHeader,
        destination: Option<PathBuf>,
    ) -> Self {
        Self {
            header,
            id,
            seeders,
            leechers,
            announce,
            info_hash,
            piece_hash,
            piece_length,
            length,
            name,
            destination,
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

    async fn check_piece(&self, index: u64) -> Result<bool, Box<dyn Error>> {
        let piece = self.piece_hash.get(index as usize).unwrap();
        Ok(true)
    }
}
#[derive(Deserialize, Serialize, Debug)]
pub enum Request {
    List(ListRequest),
    Torrent(TorrentRequest),
    Piece(PieceRequest),
    ConnectionRequest(ConnectionRequest),
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct TorrentRequest(pub String);
#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct TorrentResponse(pub Vec<u8>);

#[derive(Deserialize, Serialize, Debug)]
pub struct ConnectionRequest {}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct PieceRequest(pub String);
#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct PieceResponse(pub Vec<HashSet<String>>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListRequest {
    pub info_hashes: Vec<InfoHash>,
    downloaded: u64,
    left: u64,
    uploaded: u64,
    // event: SwarmEvent<>,
    ip_address: u32,
    key: u32,
    num_want: i32,
    port: u16,
}

#[derive(Ord, PartialOrd, Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct InfoHash {
    hash: [u8; 20],
}

impl std::fmt::Display for InfoHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rx: [u8; 20] = serde_bencode::from_bytes(&self.hash).expect("failed to serialize");
        write!(f, "{:?}", std::str::from_utf8(&rx).unwrap())
    }
}

pub fn pack<T: Serialize, W: std::io::Write>(w: &mut W, t: &T) -> Result<Vec<u8>, ()> {
    let bytes = serde_bencode::to_bytes(t).unwrap();
    match w.write_all(&bytes) {
        Ok(_) => Ok(bytes),
        Err(_) => Err(()),
    }
}

pub fn unpack<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, Box<dyn Error>> {
    let t = serde_bencode::from_bytes::<T>(bytes)?;
    Ok(t)
}

#[derive(Serialize, Deserialize, Display)]
pub enum Sources {}

#[derive(Debug)]
pub enum ChannelRequest {}

mod tests {

    #[tokio::test]
    async fn test_bincode_serialize() {
        let buffer = [0u8; 20];
        let infohash = InfoHash { hash: buffer };

        // assert!(pack(&infohash).is_ok());
    }
    async fn test_file_new() {
        // let destination_dir = Some("/Downloads".parse::<PathBuf>().unwrap());
        // let file = File::new(
        //     Some("http://tracker.com".to_string()),
        //     vec![0; 20],
        //     vec![0; 20],
        //     10,
        //     100,
        //     "test".to_string(),
        //     RequestHeader {
        //         request: Request::Torrent(TorrentRequest("test_torrent".to_string())),
        //     },
        //     destination_dir,
        // );
        // assert_eq!(file.length, 100);
    }
    async fn test_tracker_URL() {
        // let peer_id = PeerId::random();
        // let destination_dir = Some("/Downloads".parse::<PathBuf>().unwrap());

        // let file = File::new(
        //     Some("http://tracker.com".to_string()),
        //     vec![0; 20],
        //     vec![0; 20],
        //     10,
        //     100,
        //     "test".to_string(),
        //     RequestHeader {
        //         request: Request::Torrent(TorrentRequest("test_torrent".to_string())),
        //     },
        //     destination_dir,
        // );
        // let url = file.build_tracker_URL(peer_id, 9091).await.unwrap();
        // tracing::info!("url: {}", url);
        // let expected = format!(
        //     "http://tracker.com/?peer_id={}&port=9091&uploaded=0&downloaded=0&left=100",
        //     peer_id
        // );
        // assert_eq!(
        //     file.build_tracker_URL(peer_id, 9091)
        //         .await
        //         .unwrap()
        //         .to_string(),
        //     expected.to_string(),
        // );
    }
}
