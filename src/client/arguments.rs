use clap::{Parser, ValueEnum};
use libp2p::core::Multiaddr;


use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::sync::oneshot::{Sender};

use std::collections::{HashSet};
#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    pub host: String,
    #[arg(long)]
    pub ip: String,
    pub port: u16,
    #[arg(value_enum)]
    pub mode: Mode,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Mode {
    ClientMode,
    DiscoveryMode,
    UploadingMode,
    ServerMode,
    StreamingMode,
    TrackingMode,
}

pub enum Command {
    ListenCommand {
        addr: Multiaddr,
    },
    DialCommand {
        peer_id: PeerId,
        addr: Multiaddr,
        tx: Sender<Result<(), Box<dyn Error + Send>>>,
    },
    GetPeersCommand {
        file_name: String,
        tx: Sender<HashSet<PeerId>>,
    },
    GetFileCommand {
        file_name: String,
        peer: PeerId,
        tx: Sender<Result<Vec<u8>, Box<dyn Error + Send>>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FileRequest(String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileResponse(Vec<u8>);
