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

pub fn get_cmds() -> clap::Command {
    use crate::config::VERSION_STR;
    use clap::{Arg, Command};
    Command::new("jubjub")
        .version(VERSION_STR)
        .author("nuts-rice and contributors")
        .about("bbyjubjub is a torrent client. Enjoy ^_^")
        .arg(
            Arg::new("max_peers")
                .short('m')
                .long("max-peers")
                .num_args(1)
                .help("Maximum number of peers to connect to")
                .default_value("10")
                .conflicts_with("config"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .num_args(1..)
                .default_value("config.toml")
                .conflicts_with("max_peers")
                .help("TOML config file for jubjub"),
        )
        .arg(
            Arg::new("address")
                .long("address")
                .short('a')
                .num_args(1..)
                .default_value("127.0.0.1")
                .help("address to listen to"),
        )
        .arg(
            Arg::new("port")
                .long("address")
                .short('a')
                .num_args(1..)
                .default_value("3001")
                .help("port to listen to"),
        )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FileRequest(String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileResponse(Vec<u8>);
