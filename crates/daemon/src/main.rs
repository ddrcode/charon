pub mod actor;
pub mod broker;
pub mod config;
pub mod daemon;
pub mod devices;
pub mod domain;
pub mod error;

use std::fs::read_to_string;

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
    config::CharonConfig,
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
        .with_config(get_config().unwrap_or(CharonConfig::default()))
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

fn get_config() -> Result<CharonConfig, anyhow::Error> {
    fn try_get_config() -> Result<CharonConfig, anyhow::Error> {
        // use ::config::{File, FileFormat};
        // let settings = Config::builder()
        //     .add_source(File::from_str(
        //         &format!("{}/charon/charon", std::env::var("XDG_CONFIG_HOME")?),
        //         FileFormat::Toml,
        //     ))
        //     .build()?;
        // let config = settings.try_deserialize::<CharonConfig>()?;

        let c = read_to_string(&format!(
            "{}/charon/charon.toml",
            std::env::var("XDG_CONFIG_HOME")?
        ))?;
        let config: CharonConfig = toml::from_str(&c)?;

        info!("Using config file");
        Ok(config)
    }

    let config_result = try_get_config();
    if let Err(ref err) = config_result {
        tracing::error!("Error processing config file: {}", err);
    }
    config_result
}
