pub mod client;
pub mod config;
pub mod db;
pub mod network;
pub mod parser;
pub mod peer;
pub mod types;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    client::cli::cli();
    let _bytes = [2., 13., 8., 1., 2., 3., 4., 5., 6., 7., 8., 9., 10.];
    let mut _bytes = [2., 13., 8., 1., 2., 4., 4., 2., 6., 2., 8., 2., 2.];
    //moved to network
    let _network = network::new().await;
    unimplemented!()
}
