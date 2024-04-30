use crate::client::arguments::*;
use clap::Parser;


pub fn cli() {
    let _cli = Args::parse();
    // match cli.cmd {
    //     Command::DecodeCommand { val } => {
    //         info!("Decode value: {}", val);
    //     }
    //     Command::TorrentInfoCommand { torrent } => {
    //         info!("Torrent info on torrent: {:?}", torrent);
    //     }
    //     Command::DialCommand {
    //         peer_id,
    //         torrent,
    //         addr,
    //     } => {
    //         info!(
    //             "Dialing... peer_id: {}, torrent: {:?}, addr: {}",
    //             peer_id, torrent, addr
    //         );
    //     }
    //     Command::ListenCommand { addr } => {
    //         info!("listening on adddr: {}", addr);
    //     }
    //     Command::GetFileCommand {
    //         output,
    //         torrent,
    //         peer,
    //     } => {
    //         info!(
    //             "Get file command selected with output: {:?}, torrent: {:?}, peer: {}",
    //             output, torrent, peer
    //         );
    //     }
    //     Command::GetPeersCommand { torrent } => {
    //         info!("Get peers command selected with torrent: {:?}", torrent);
    //     } // Mode::ServerMode => {
    //       //     info!("Server mode selected");
    //       // }
    //       // Mode::ClientMode => {
    //       //     info!("Client mode selected");
    //       // }
    //       // Mode::DiscoveryMode => {
    //       //     info!("Discovery mode selected");
    //       // }
    //       // Mode::TrackingMode => {
    //       //     info!("Tracking mode selected");
    //       // }
    //       // Mode::StreamingMode => {
    //       //     info!("Streaming mode selected");
    //       // }
    //       // Mode::UploadingMode => {
    //       //     info!("Uploading mode selected");
    //       // }
    // }
}
