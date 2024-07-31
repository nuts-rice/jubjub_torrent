pub mod client;
pub mod config;
pub mod db;
pub mod metrics;
pub mod network;
pub mod parser;
pub mod peer;
pub mod types;

use crate::client::arguments::{get_cmds, Settings};
use eframe::egui;
use libp2p::metrics::Registry;
use metrics::{setup_tracing, MetricServer};
use std::error::Error;
use std::sync::{Arc, RwLock};

pub struct App {
    torrents: Vec<String>,
    config: Arc<RwLock<Settings>>,
    torrent_files: Vec<egui::DroppedFile>,
    torrent_file_path: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        App {
            torrents: vec!["example_torrent.torrent".to_string()],
            torrent_files: vec![],
            torrent_file_path: None,
            config: Arc::new(RwLock::new(Settings::default())),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let download_dir = {
            let config_guard = self.config.read().unwrap();
            config_guard.download_dir.clone()
        };
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Torrents");
            if ui.button("Add torrent").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.torrent_file_path = Some(path.display().to_string());
                }
            }
            if let Some(torrent_path) = &self.torrent_file_path {
                ui.horizontal(|ui| {
                    ui.label("Torrent file:");
                    ui.monospace(torrent_path)
                });
            }
            if !self.torrent_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");
                    for file in &self.torrent_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "Unknown".to_owned()
                        };
                        let mut additional_info = vec![];
                        if !file.mime.is_empty() {
                            additional_info.push(format!("type: {}", file.mime));
                        }
                        if let Some(bytes) = &file.bytes {
                            additional_info.push(format!("{} bytes", bytes.len()));
                        }
                        if !additional_info.is_empty() {
                            info += &format!(" ({})", additional_info.join(", "));
                        }
                        ui.label(info);
                    }
                });
            }
        });
        // preview_files(ctx);
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.torrent_files.clone_from(&i.raw.dropped_files);
            }
        });
    }
}

fn preview_files(ctx: &egui::Context) {
    unimplemented!()
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_rwlock = Arc::new(RwLock::new(Settings::new(get_cmds()).await));
    let metrics_registry = Registry::default();
    let registry_rwlock = Arc::new(RwLock::new(metrics_registry));
    let metrics = MetricServer::new(registry_rwlock.clone(), config_rwlock.clone());

    //moved to network
    let (_network_client, _network_events, network_session) =
        network::new(config_rwlock.clone(), metrics.clone())
            .await
            .unwrap();
    tokio::spawn(network_session.run());
    setup_tracing();
    tokio::spawn(metrics::metrics_server(metrics));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "jubjub_torrent",
        options,
        Box::new(|cc| {
            // egui_extras
            Box::<App>::default()
        }),
    );
    Ok(())
}
