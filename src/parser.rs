use async_trait::async_trait;
use std::error::Error;
use serde::{Deserialize, Serialize};
pub fn chi_squared_test(observed: &Vec<f64>, expected: &Vec<f64>) -> f64 {
    let mut chi_squared = 0.0;
    for i in 0..observed.len() {
        chi_squared += (observed[i] - expected[i]).powi(2) / expected[i];
    }
    chi_squared
}

#[derive(Deserialize, Serialize, )]
struct RequestHeader{}


#[derive( Deserialize, Serialize, )]
pub struct File {
    header: RequestHeader,
    Announce: Option<String>,
    InfoHash: Vec<u8>,
    PieceHash: Vec<u8>,
    PieceLength: u64,
    Length: u64,
    Name: String,
}

impl File {
    pub fn new(announce: Option<String>, info_hash: Vec<u8>, piece_hash: Vec<u8>, piece_length: u64, length: u64, name: String, request_header: RequestHeader) -> Self {
        Self {
            header: request_header,
            Announce: announce,
            InfoHash: info_hash,
            PieceHash: piece_hash,
            PieceLength: piece_length,
            Length: length,
            Name: name,
        }
    }
            async fn from_bytes(bytes: &[u8]) -> Self {
        unimplemented!()
    }
    async fn build_tracker_URL(&self) -> Result<String, Box<dyn Error>> {
        unimplemented!()
    }


}

#[async_trait]
pub trait TorrentFile {
    type File; 
    async fn from_file(file: &str) -> Self;
    async fn from_bytes(bytes: &[u8]) -> Self;
    async fn build_tracker_URL(&self) -> Result<String, Box<dyn Error>>;
}



mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tracker_URL() {
        let file = File::new(Some("http://tracker.com".to_string()), vec![0; 20], vec![0; 20], 10, 100, "test".to_string(), RequestHeader{});
        assert_eq!(file.build_tracker_URL().await.unwrap(), "http://tracker.com".to_string());
    }
}
