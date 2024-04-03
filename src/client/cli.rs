use clap::{Parser};
use tracing::info;
use crate::client::arguments::*;

pub fn cli() {
   let cli = Args::parse(); 
   match cli.mode {
    Mode::ServerMode => {
        info!("Server mode selected");
    },
    Mode::ClientMode => {
        info!("Client mode selected");
    },
    Mode::DiscoveryMode => {
        info!("Discovery mode selected");
    },
    Mode::TrackingMode => {
        info!("Tracking mode selected");
    },
    Mode::StreamingMode => {
        info!("Streaming mode selected");
    },
    Mode::UploadingMode => {
        info!("Uploading mode selected");
    },       
       
   }
}
