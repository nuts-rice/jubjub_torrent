use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Torrent {
    pub id: String,
    pub name: Option<String>,
    pub path: String,
    pub priority: u8,
    pub created_at: u128,
    pub updated_at: u128,
}
