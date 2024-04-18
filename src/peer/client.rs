use libp2p::futures::Stream;
use libp2p::{StreamProtocol, Swarm};
use tokio::sync::mpsc::{Sender, Receiver};
use std::error::Error;
use serde_json as json;
use serde_bencode as bencode;
use crate::client::arguments::Command;
use crate::parser::TorrentFile;
use crate::types::{Event,Node};
pub struct Client {
    pub tx: Sender<Command>,

}

#[repr(C)]
pub struct Handshake {
    pub len: u8,
    pub bittorent: [u8; 19],
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Handshake {
            len: 19,
            bittorent: *b"BitTorrent protocol",
            reserved: [0; 8],
            info_hash,
            peer_id,
        }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unimplemented!()
    }
}

impl Node for Client {
    fn get_peer_id(&self) -> u32 {
        unimplemented!()
    }
        
}

impl Client {
    async fn handle_command(&mut self, command: Command) -> Result<(), Box<dyn Error>> {
        match command {
            Command::DecodeCommand { val } => {
                unimplemented!()
            }
            Command::TorrentInfoCommand { torrent } => {
                unimplemented!()
            }
            Command::GetFileCommand{output, torrent, peer,} => {
                unimplemented!()
            }
            Command::GetPeersCommand { torrent}  => {
                unimplemented!()
            }
            Command::DialCommand { peer_id, torrent, addr, } => {
                unimplemented!()
            }
            Command::ListenCommand { addr } => {
                unimplemented!()
            }

        }
    }
    // pub async fn new(seed: Option<u8>) -> Result<(Client, impl Stream<Item = Event> ,Session), Box<dyn Error>> {
    //     unimplemented!()
    // }
    pub async fn get_file(&self, file: String) -> Result<(), Box<dyn Error>> {
        let id = self.get_peer_id();
        Ok(())
    }

    pub fn decode_value(val: String) -> (json::Value, String) {
        let serialized  = bencode::to_string(&val).unwrap();
        let res = json::Value::String(val.to_string());
        (res, serialized)
    }




    // pub async fn build_tracker(&self, port: u16, file: crate::parser::File) -> Result<String, Box<dyn Error>> {
    //     let id = self.get_peer_id();
    //     unimplemented!()
    // }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;
    use tracing_test::traced_test;
    #[test]
    #[traced_test]
    fn test_decode_value() {
        let (val, serialized) = Client::decode_value("d4:spam3:egge".to_string());
        info!("decoded bencode val: {:?}, serialized: {:?}", val, serialized);
            
    }
}
