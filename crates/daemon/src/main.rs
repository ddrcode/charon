pub mod actor;
pub mod broker;
pub mod config;
pub mod daemon;
pub mod devices;
pub mod domain;
pub mod error;
pub mod processor;
pub mod util;

use anyhow;
use charon_lib::event::Topic as T;
use std::{fs::read_to_string, path::PathBuf};
use tokio::{self, signal::unix};
use tracing::{debug, info, warn};
use tracing_subscriber::FmtSubscriber;

use crate::{
    actor::{
        ipc_server::IPCServer, key_writer::KeyWriter, power_manager::PowerManager,
        telemetry::Telemetry, typing_stats::TypingStats, typist::Typist,
    },
    config::CharonConfig,
    daemon::Daemon,
    processor::{KeyEventToUsbReport, SystemShortcut},
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    init_logging();

    let config = get_config().expect("Failed loading config file");
    let mut daemon = Daemon::new();
    daemon
        .with_config(config.clone())
        .add_scanners(&[T::System])
        .add_actor::<Typist>(&[T::System, T::TextInput])
        .add_actor::<KeyWriter>(&[T::System, T::KeyOutput])
        .add_actor::<TypingStats>(&[T::System, T::KeyInput])
        .add_actor::<IPCServer>(&[T::System, T::KeyInput, T::Stats, T::Monitoring])
        .add_pipeline(
            "PassThroughPipeline",
            &[T::System, T::KeyInput],
            vec![
                Box::new(KeyEventToUsbReport::new()),
                Box::new(SystemShortcut::new()),
            ],
        )
        .add_actor_conditionally::<PowerManager>(
            config.sleep_script.is_some() && config.awake_script.is_some(),
            &[T::System, T::KeyInput],
        )
        .add_actor_conditionally::<Telemetry>(
            config.enable_telemetry,
            &[T::System, T::Telemetry, T::KeyInput],
        );

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

fn init_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .compact()
        .pretty()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");
}
