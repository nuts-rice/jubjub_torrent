use std::path::PathBuf;

use libp2p::{
    identity::{Keypair}, // mdns
};
use crate::client::arguments::Command;
use tokio::sync::oneshot;

use super::client::Client;

async fn start_discovery(_id_key: Keypair, client: Client, file: PathBuf   ) {
    unimplemented!()
    // let (tx, mut rx) = oneshot::channel();
    // client.tx.send(Command::GetPeersCommand{torrent: file, }).await.expect("Failed to send command to client : GetPeersCommand");
    // rx.await.expect("Failed to get response from client : GetPeersCommand");
}



#[cfg(test)]
mod tests {
   use super::*; 

   #[tokio::test]
   async fn test_discovery_mode(){
   unimplemented!()

   }
}
