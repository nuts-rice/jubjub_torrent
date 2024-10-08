use clap::{ArgMatches, Command, Parser, ValueEnum};
use futures::channel::oneshot;
use libp2p::core::Multiaddr;
use libp2p::PeerId;
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
    pub address: Multiaddr,
    pub socket_workers: usize,
}

#[derive(Debug, Clone)]
pub struct IPFSSettings {
    pub address: Multiaddr,
    pub socket_workers: usize,
    pub path: String,
    pub timeout: u64,
}

#[derive(Debug, Clone)]
pub struct WSSettings {
    pub address: Multiaddr,
    pub socket_workers: usize,
}
#[derive(Debug, Clone)]
pub struct MetricsSettings {
    pub socket_addr: SocketAddr,
    pub route: String,
    pub update_interval: u64,
}

impl Default for TcpSettings {
    fn default() -> Self {
        Self {
            address: "/ip4/127.0.0.1:3001".parse::<Multiaddr>().unwrap(),
            socket_workers: 1,
        }
    }
}
impl Default for WSSettings {
    fn default() -> Self {
        Self {
            address: ("/ip4/127.0.0.1:3000".parse::<Multiaddr>().unwrap()),
            socket_workers: 1,
        }
    }
}

impl Default for MetricsSettings {
    fn default() -> Self {
        Self {
            socket_addr: ("127.0.0.1:9091".parse::<SocketAddr>().unwrap()),
            route: ("/metrics".to_string()),
            update_interval: 5,
        }
    }
}

impl Default for IPFSSettings {
    fn default() -> Self {
        Self {
            address: ("/ip4/127.0.0.1".parse::<Multiaddr>().unwrap()),
            path: ("/ip4".to_string()),
            timeout: 10,
            socket_workers: 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub tcp: TcpSettings,
    pub ws: WSSettings,
    pub metrics: MetricsSettings,
    pub ipfs: IPFSSettings,
    pub max_peers: usize,
    pub download_dir: PathBuf,
    secret_key: Option<u8>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tcp: TcpSettings::default(),
            ws: WSSettings::default(),
            metrics: MetricsSettings::default(),
            ipfs: IPFSSettings::default(),
            max_peers: 10,
            download_dir: "~/Downloads".parse::<PathBuf>().unwrap(),
            secret_key: None,
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
        let secret_key = jubjub_table
            .get("key")
            .expect("Missing key field")
            .as_str()
            .expect("Invalid download_dir field")
            .parse::<u8>()
            .unwrap();
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
        let _address = address
            .parse::<SocketAddr>()
            .expect("\x1b[31mErr:\x1b[0m Could not address to SocketAddr!");
        let tcp = TcpSettings {
            address: tcp_table
                .get("address")
                .expect("Missing address field")
                .as_str()
                .unwrap()
                .parse::<Multiaddr>()
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
                .parse::<Multiaddr>()
                .unwrap(),

            socket_workers: ws_table
                .get("socket_workers")
                .expect("Missing socket_workers field")
                .as_integer()
                .expect("Invalid socket_workers field") as usize,
        };
        // let ipfs_table = parsed
        //     .get("ipfs")
        //     .expect("Missing ipfs field")
        //     .as_table()
        //     .expect("Invalid ipfs field");
        // let ipfs = IPFSSettings {
        //     address: ipfs_table
        //         .get("address")
        //         .expect("Missing address field")
        //         .as_str()
        //         .unwrap()
        //         .parse::<Multiaddr>()
        //         .unwrap(),
        //     socket_workers: ipfs_table
        //         .get("socket_workers")
        //         .expect("Missing socket_workers field")
        //         .as_integer()
        //         .expect("Invalid socket_workers field") as usize,
        //     path: ipfs_table
        //         .get("path")
        //         .expect("Missing path field")
        //         .as_str()
        //         .unwrap()
        //         .to_string(),
        //     timeout: ipfs_table
        //         .get("timeout")
        //         .expect("Missing timeout field")
        //         .as_integer()
        //         .expect("Invalid timeout field") as u64,
        // };
        let ipfs = IPFSSettings {
            path: "/ipfs/".to_string(),
            address: "/ip4/127.0.0.1".parse::<Multiaddr>().unwrap(),
            timeout: 10,
            socket_workers: 04,
        };
        let metrics_table = parsed
            .get("metrics")
            .expect("Missing metrics field")
            .as_table()
            .expect("Invalid metrics field");
        let metrics = MetricsSettings {
            socket_addr: metrics_table
                .get("address")
                .expect("Missing address field")
                .as_str()
                .unwrap()
                .parse::<SocketAddr>()
                .unwrap(),

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
                .unwrap() as u64,
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
            ipfs,
            max_peers,
            download_dir,
            secret_key: Some(secret_key),
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
            .parse::<Multiaddr>()
            .expect("Invalid address or port!");
        let tcp = TcpSettings {
            address: address.clone(),
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
                socket_addr: address
                    .parse::<SocketAddr>()
                    .expect("Invalid metrics address"),
                route: route.to_string(),
                update_interval: interval,
            }
        } else {
            MetricsSettings {
                route: "/metrics".to_string(),
                socket_addr: "::1:9091".parse::<SocketAddr>().unwrap(),
                update_interval: 10,
            }
        };
        let ipfs_enabled = matches.get_occurrences::<String>("ipfs").is_some();
        // let ipfs = if ipfs_enabled {
        //     let address = matches
        //         .get_one::<String>("ipfs_address")
        //         .expect("Invalid ipfs_address");
        //     let timeout = matches
        //         .get_one::<String>("timeout")
        //         .expect("Invalid timeout")
        //         .parse::<u64>()
        //         .expect("Invalid timeout");
        //     let path = matches.get_one::<String>("path").expect("Invalid path");
        //     let socket_workers = matches
        //         .get_one::<String>("socket_workers")
        //         .expect("Invalid socket_workers")
        //         .parse::<usize>()
        //         .expect("Invalid socket_workers");
        //     let timeout = matches
        //         .get_one::<String>("timeout")
        //         .expect("Invalid timeout")
        //         .parse::<u64>()
        //         .expect("Invalid timeout");
        //     IPFSSettings {
        //         address: address.parse::<Multiaddr>().expect("Invalid ipfs address"),
        //         socket_workers,
        //         path: path.to_string(),
        //         timeout,
        //     }
        // } else {
        let ipfs = IPFSSettings {
            address: "/ip4/".parse::<Multiaddr>().unwrap(),

            socket_workers: 04,
            path: "/ipfs/".to_string(),
            timeout: 10,
            // }
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
        let secret_key = matches
            .get_one::<String>("key>")
            .expect("Invalid secret key")
            .parse::<u8>()
            .expect("Invalid secret key");

        Settings {
            tcp,
            ws,
            ipfs,
            metrics,
            max_peers,
            download_dir,
            secret_key: Some(secret_key),
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

pub fn execute_cmd(_tx: serde_json::Value) -> Result<(), Box<dyn Error>> {
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
            Arg::new("key")
                .short('k')
                .long("key")
                .help("Secret key for encryption"),
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
                .default_value("/ip4/127.0.0.1:3001")
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
                .default_value("/ip4/127.0.0.1:3002")
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
            Arg::new("metrics_socket_address")
                .long("metrics_socket_address")
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
        .arg(
            Arg::new("ipfs_address")
                .long("ipfs_address")
                .num_args(1..)
                .default_value("/ip4/127.0.0.1:5001")
                .help("ipfs address to listen to"),
        )
        .arg(
            Arg::new("ipfs_timeout")
                .long("ipfs_timeout")
                .num_args(1..)
                .default_value("10")
                .help("ipfs timeout in seconds "),
        )
        .arg(
            Arg::new("ipfs_socket_worker")
                .long("ipfs_socket_worker")
                .num_args(1..)
                .default_value("10")
                .help("number of ipfs socket workers "),
        )
        .arg(
            Arg::new("ipfs_path")
                .long("ipfs_path")
                .num_args(1..)
                .default_value("10")
                .help("ipfs path to listen to"),
        )
}

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// struct FileRequest(String);
// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub(crate) struct FileResponse(Vec<u8>);
