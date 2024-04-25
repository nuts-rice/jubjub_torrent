use clap::{Parser, Subcommand, ValueEnum};
use libp2p::core::Multiaddr;

use libp2p::PeerId;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, author, about)]
pub struct Args {
    #[arg(long)]
    pub host: String,
    #[arg(long)]
    pub ip: String,
    pub port: u16,
    #[command(subcommand)]
    pub cmd: Command,
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

#[derive(Subcommand, Debug)]
pub enum Command {
    DecodeCommand {
        val: String,
    },
    TorrentInfoCommand {
        torrent: PathBuf,
    },
    ListenCommand {
        addr: Multiaddr,
        // tx: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    DialCommand {
        peer_id: PeerId,
        torrent: PathBuf,
        addr: Multiaddr,
    },
    GetPeersCommand {
        torrent: PathBuf,
    },
    GetFileCommand {
        #[arg(short)]
        output: PathBuf,
        torrent: PathBuf,
        peer: PeerId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FileRequest(String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileResponse(Vec<u8>);
