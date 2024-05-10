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
    pub download_dir: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tcp: TcpSettings::default(),
            ws: WSSettings::default(),
            metrics: MetricsSettings::default(),
            max_peers: 10,
            download_dir: "~/Downloads".parse::<PathBuf>().unwrap(),
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
        Settings::create_from_matches(matches)
    }

    async fn create_from_file(file: String) -> Settings {
        use toml::Value;
        let parsed = file.parse::<Value>().expect("Invalid config file");
        let jubjub_table = parsed
            .get("jubjub")
            .expect("Missing jubjub field")
            .as_table()
            .expect("Invalid jubjub field");
        let address = jubjub_table
            .get("address")
            .expect("Missing address field")
            .as_str()
            .expect("Invalid address field");
        let download_dir = jubjub_table
            .get("download_dir")
            .expect("Missing download_dir field")
            .as_str()
            .expect("Invalid download_dir field")
            .parse::<PathBuf>()
            .unwrap();
        let tcp_table = parsed
            .get("tcp")
            .expect("Missing tcp field")
            .as_table()
            .expect("Invalid tcp field");
        let ws_table = parsed
            .get("ws")
            .expect("Missing ws field")
            .as_table()
            .expect("Invalid ws field");
        let port = 3001;
        let address = address.replace("localhost", "127.0.0.1");
        let address = if address.contains(':') {
            address.to_string()
        } else {
            format!("{}:{}", address, port)
        };
        let address = address
            .parse::<SocketAddr>()
            .expect("\x1b[31mErr:\x1b[0m Could not address to SocketAddr!");
        let tcp = TcpSettings {
            address: tcp_table
                .get("address")
                .expect("Missing address field")
                .as_str()
                .unwrap()
                .parse::<SocketAddr>()
                .unwrap(),
            socket_workers: tcp_table
                .get("socket_workers")
                .expect("Missing socket_workers field")
                .as_integer()
                .expect("Invalid socket_workers field") as usize,
        };
        let ws = WSSettings {
            address: ws_table
                .get("address")
                .expect("Missing address field")
                .as_str()
                .unwrap()
                .parse::<SocketAddr>()
                .unwrap(),

            socket_workers: ws_table
                .get("socket_workers")
                .expect("Missing socket_workers field")
                .as_integer()
                .expect("Invalid socket_workers field") as usize,
        };
        let metrics_table = parsed
            .get("metrics")
            .expect("Missing metrics field")
            .as_table()
            .expect("Invalid metrics field");
        let metrics = MetricsSettings {
            address: metrics_table
                .get("address")
                .expect("Missing address field")
                .as_str()
                .unwrap()
                .parse::<SocketAddr>()
                .expect("Invalid address field"),
            route: metrics_table
                .get("route")
                .expect("Missing route field")
                .as_str()
                .expect("Invalid route field")
                .to_string(),
            update_interval: metrics_table
                .get("update_interval")
                .expect("Missing update_interval field")
                .as_integer()
                .expect("Invalid update_interval field") as u64,
        };
        let max_peers = jubjub_table
            .get("max_peers")
            .expect("Missing max_peers field")
            .as_integer()
            .expect("Invalid max_peers field") as usize;
        Settings {
            tcp,
            ws,
            metrics,
            max_peers,
            download_dir,
        }
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
        let tcp = TcpSettings {
            address,
            socket_workers: 1,
        };
        let ws = WSSettings {
            address,
            socket_workers: 1,
        };
        let enabled = matches.get_occurrences::<String>("metrics").is_some();
        let metrics = if enabled {
            let address = matches
                .get_one::<String>("metrics_address")
                .expect("Invalid metrics_address");
            let interval = matches
                .get_one::<String>("update_interval")
                .expect("Invalid update_interval")
                .parse::<u64>()
                .expect("Invalid update_interval");
            let route = matches.get_one::<String>("route").expect("Invalid route");
            MetricsSettings {
                address: address
                    .parse::<SocketAddr>()
                    .expect("Invalid metrics address"),
                route: route.to_string(),
                update_interval: interval,
            }
        } else {
            MetricsSettings {
                route: "/metrics".to_string(),
                address: "::1:9091".parse::<SocketAddr>().unwrap(),
                update_interval: 10,
            }
        };
        let max_peers = matches
            .get_one::<String>("max_peers")
            .expect("Invalid max_peers")
            .parse::<usize>()
            .expect("Invalid max_peers");
        let download_dir = matches
            .get_one::<String>("download_dir")
            .expect("Invalid download dir")
            .parse::<PathBuf>()
            .expect("Invalid download dir");

        Settings {
            tcp,
            ws,
            metrics,
            max_peers,
            download_dir,
        }
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

pub fn execute_cmd(tx: serde_json::Value) -> Result<(), Box<dyn Error>> {
    unimplemented!()
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
                .default_value("127.0.0.1:3000")
                .help("address to listen to"),
        )
        .arg(
            Arg::new("tcp_address")
                .long("tcp_address")
                .num_args(1..)
                .default_value("127.0.0.1:3001")
                .help("tcp address to listen to"),
        )
        .arg(
            Arg::new("tcp_socket_workers")
                .long("tcp_sockets")
                .num_args(1..)
                .default_value("1")
                .help("number of tcp socket workers"),
        )
        .arg(
            Arg::new("ws_address")
                .long("ws_address")
                .num_args(1..)
                .default_value("127.0.0.1:3002")
                .help("ws address to listen to"),
        )
        .arg(
            Arg::new("ws_sockets")
                .long("ws_sockets")
                .num_args(1..)
                .default_value("1")
                .help("number of ws socket workers"),
        )
        .arg(
            Arg::new("metrics_address")
                .long("metrics_address")
                .num_args(1..)
                .default_value("127.0.0.1:9091")
                .help("Address to listen to for the metrics"),
        )
        .arg(
            Arg::new("metrics_route")
                .long("metrics_route")
                .num_args(1..)
                .default_value("/metrics")
                .help("route to listen to for the metrics"),
        )
        .arg(
            Arg::new("metrics_interval")
                .long("metrics_interval")
                .num_args(1..)
                .default_value("10")
                .help("Interval in seconds to collect metrics"),
        )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FileRequest(String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileResponse(Vec<u8>);
