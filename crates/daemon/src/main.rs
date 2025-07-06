pub mod actor;
pub mod broker;
pub mod daemon;
pub mod domain;
pub mod error;

use anyhow;
use tokio::{self, signal::unix};
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use crate::daemon::Daemon;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let mut daemon = Daemon::new();

    let mut sigterm = unix::signal(unix::SignalKind::terminate())?;

    tokio::select! {
        _ = daemon.start() => {},
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down gracefully...");
            daemon.stop().await;
        },
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down");
            daemon.stop().await;
        }
    }

    daemon.shutdown().await;

    info!("Charon says goodbye. Hades is waiting...");
    Ok(())
}
