pub mod actor;
pub mod broker;
pub mod config;
pub mod daemon;
pub mod devices;
pub mod domain;
pub mod error;

use anyhow;
use charon_lib::event::Topic as T;
use std::{fs::read_to_string, path::PathBuf};
use tokio::{self, signal::unix};
use tracing::{debug, info, warn};
use tracing_subscriber::FmtSubscriber;

use crate::{
    actor::{
        ipc_server::IPCServer, key_scanner::KeyScanner, key_writer::KeyWriter,
        passthrough::PassThrough, telemetry::Telemetry, typing_stats::TypingStats, typist::Typist,
    },
    config::{CharonConfig, InputConfig},
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
        .with_config(get_config().expect("Failed loading config file"))
        .add_actor("KeyScanner", KeyScanner::spawn, &[T::System])
        .add_actor("PassThrough", PassThrough::spawn, &[T::System, T::KeyInput])
        .add_actor("Typist", Typist::spawn, &[T::System, T::TextInput])
        .add_actor("KeyWriter", KeyWriter::spawn, &[T::System, T::KeyOutput])
        .add_actor("TypingStats", TypingStats::spawn, &[T::System, T::KeyInput])
        .add_actor(
            "Telemetry",
            Telemetry::spawn,
            &[T::System, T::Monitoring, T::KeyInput],
        )
        .add_actor(
            "IPCServer",
            IPCServer::spawn,
            &[T::System, T::KeyInput, T::Stats, T::Monitoring],
        )
        .update_config(|config| {
            config.keyboard = InputConfig::Name("usb-Keychron_Keychron_Q10-event-if02".into())
        })
        .add_actor("KnobScanner", KeyScanner::spawn, &[T::System]);

    let mut sigterm = unix::signal(unix::SignalKind::terminate())?;

    tokio::select! {
        _ = daemon.run() => {},
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
            daemon.stop().await;
        },
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down...");
            daemon.stop().await;
        }
    }

    daemon.shutdown().await;

    info!("Charon says goodbye. Hades is waiting...");
    Ok(())
}

fn get_config() -> Result<CharonConfig, anyhow::Error> {
    let mut path = PathBuf::new();
    path.push(std::env::var("XDG_CONFIG_HOME")?);
    path.push("charon/charon.toml");

    if !path.exists() {
        warn!(
            "Couldn't find config file at {:?}. Starting with default configuration",
            path
        );
        return Ok(CharonConfig::default());
    }

    debug!("Found config file: {:?}", path);
    let config_str = read_to_string(path)?;
    let config: CharonConfig = toml::from_str(&config_str)?;

    Ok(config)
}
