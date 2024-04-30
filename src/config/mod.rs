use serde::Deserialize;

pub const VERSION_STR: &str = env!("CARGO_PKG_VERSION");

pub mod tcp_config;
pub mod ws_config;
pub use tcp_config::TCPConfig;
pub use ws_config::WSConfig;
#[derive(Deserialize)]
pub struct Config {
    tcp: TCPConfig,
    ws: WSConfig,
}
