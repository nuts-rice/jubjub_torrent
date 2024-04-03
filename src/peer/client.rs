use tokio::sync::mpsc::{Sender};
pub struct Client {
    tx: Sender<String>,
}
