use clap::{ArgMatches, Command, Parser, ValueEnum};
use futures::channel::oneshot;
use libp2p::core::Multiaddr;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, author, about)]
pub struct Args {
    #[arg(long)]
    pub host: String,
    #[arg(long)]
    pub ip: String,
    //TODO: move commands somewhere else
    // #[command(subcommand)]
    // pub cmd: Command,
}
#[derive(Debug, Clone)]
pub struct TcpSettings {
    pub address: SocketAddr,
    pub socket_workers: usize,
}

#[derive(Debug, Clone)]
pub struct WSSettings {
    pub address: SocketAddr,
    pub socket_workers: usize,
}
#[derive(Debug, Clone)]
pub struct MetricsSettings {
    pub address: SocketAddr,
    pub route: String,
    pub update_interval: u64,
}

impl Default for TcpSettings {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:3001".parse::<SocketAddr>().unwrap(),
            socket_workers: 1,
        }
    }
}
impl Default for WSSettings {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:3000".parse::<SocketAddr>().unwrap(),
            socket_workers: 1,
        }
    }
}

impl Default for MetricsSettings {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:9091".parse::<SocketAddr>().unwrap(),
            route: "/metrics".to_string(),
            update_interval: 5,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub tcp: TcpSettings,
    pub ws: WSSettings,
    pub metrics: MetricsSettings,
    pub max_peers: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tcp: TcpSettings::default(),
            ws: WSSettings::default(),
            metrics: MetricsSettings::default(),
            max_peers: 10,
        }
    }
}

impl Settings {
    pub async fn new(matches: Command) -> Settings {
        let matches = matches.get_matches();

        // Try to open the file at the path specified in the args
        let path = matches.get_one::<String>("config").unwrap();
        let file: Option<String> = match std::fs::read_to_string(path) {
            Ok(file) => Some(file),
            Err(_) => panic!("\x1b[31mErr:\x1b[0m Error opening config file at {}", path),
        };
        if let Some(file) = file {
            tracing::info!("Using config file at {}", path);
            return Settings::create_from_file(file).await;
        }

        tracing::info!("using command line args for settings  ");
        todo!()
    }

    async fn create_from_file(file: String) -> Settings {
        unimplemented!()
    }

    fn create_from_matches(matches: ArgMatches) -> Settings {
        let address = matches
            .get_one::<String>("address")
            .expect("address is required");
        let port = matches.get_one::<String>("port").expect("Invalid port");
        let address = if address.contains(':') {
            address.to_string()
        } else {
            format!("{}:{}", address, port)
        };

        let address = address
            .parse::<SocketAddr>()
            .expect("Invalid address or port!");

        unimplemented!()
    }
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

#[derive(Debug)]
pub enum ClientCommand {
    DecodeCommand {
        val: String,
    },
    TorrentInfoCommand {
        torrent: PathBuf,
    },
    ListenCommand {
        addr: Multiaddr,
        tx: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    DialCommand {
        peer_id: PeerId,
        torrent: PathBuf,
        addr: Multiaddr,
        tx: futures::channel::oneshot::Sender<Result<(), Box<dyn std::error::Error + Send>>>,
    },
    GetPeersCommand {
        torrent: String,
        tx: futures::channel::oneshot::Sender<std::collections::HashSet<PeerId>>,
    },
    ProvideTorrent {
        file: Vec<u8>,
        channel: futures::channel::oneshot::Sender<()>,
    },
    GetFileCommand {
        output: PathBuf,
        torrent: String,
        peer: PeerId,
        tx: oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>,
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
                .long("port")
                .short('p')
                .num_args(1..)
                .default_value("3001")
                .help("port to listen to"),
        )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FileRequest(String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileResponse(Vec<u8>);
