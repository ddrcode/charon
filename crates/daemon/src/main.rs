pub mod actor;
pub mod broker;
pub mod daemon;
pub mod devices;
pub mod domain;
pub mod error;
pub mod utils;

use anyhow;
use charon_lib::event::Topic as T;
use tokio::{self, signal::unix};
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use crate::{
    actor::{
        ipc_server::IPCServer, key_scanner::KeyScanner, key_writer::KeyWriter,
        passthrough::PassThrough, typing_stats::TypingStats, typist::Typist,
    },
    daemon::Daemon,
    domain::Actor,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .compact()
        .pretty()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let mut daemon = Daemon::new();
    daemon
        .add_actor("KeyScanner", KeyScanner::spawn, &[T::System])
        .add_actor("PassThrough", PassThrough::spawn, &[T::System, T::KeyInput])
        .add_actor("Typist", Typist::spawn, &[T::System, T::TextInput])
        .add_actor("KeyWriter", KeyWriter::spawn, &[T::System, T::KeyOutput])
        .add_actor("TypingStats", TypingStats::spawn, &[T::System, T::KeyInput])
        .add_actor(
            "IPCServer",
            IPCServer::spawn,
            &[T::System, T::KeyInput, T::Stats, T::Monitoring],
        );

    let mut sigterm = unix::signal(unix::SignalKind::terminate())?;

    tokio::select! {
        _ = daemon.run() => {},
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
