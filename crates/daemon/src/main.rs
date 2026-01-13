pub mod actor;
pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod port;
pub mod processor;
pub mod util;

use charon_lib::event::{Mode, Topic as T};
use maiko::Supervisor;
use std::{fs::read_to_string, path::PathBuf, sync::Arc};
use tokio::{self, io::unix::AsyncFd, signal::unix};
use tracing::{debug, info, warn};
use tracing_subscriber::FmtSubscriber;

use crate::{
    actor::{
        KeyScanner, KeyWriter, Pipeline, PowerManager, QMK, Telemetry, TypingStats, Typist,
        ipc_server::IPCServer,
    },
    adapter::{EventDeviceUnix, HIDDeviceUnix, KeymapLoaderYaml, QmkAsyncHidDevice},
    config::CharonConfig,
    domain::{ActorState, traits::Processor},
    error::CharonError,
    port::KeymapLoader,
    processor::{KeyEventProcessor, SystemShortcutProcessor},
    util::evdev::find_input_device,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init_logging();

    let config = Arc::new(get_config().expect("Failed loading config file"));
    let state = ActorState::new(Mode::PassThrough, config.clone());
    let keymap = KeymapLoaderYaml::new(&config.keymaps_dir)
        .load_keymap(&config.host_keymap)
        .await?;

    let mut supervisor = Supervisor::default();

    for (name, config) in config.get_config_per_keyboard() {
        supervisor.add_actor(
            &format!("KeyScanner-{name}"),
            |ctx| {
                let keyboard = &config.keyboard;
                let device_path = find_input_device(keyboard)
                    .ok_or_else(|| CharonError::KeyboardNotFound(name.clone()))
                    .unwrap();
                let device = evdev::Device::open(device_path).unwrap();
                let async_dev = AsyncFd::new(device).unwrap();
                let input = EventDeviceUnix::new(async_dev);
                KeyScanner::new(ctx, state.clone(), Box::new(input), name)
            },
            &[T::System],
        )?;
    }

    supervisor.add_actor(
        "KeyWriter",
        |ctx| {
            let dev_path = config.hid_keyboard.clone();
            let dev = HIDDeviceUnix::new(&dev_path);
            KeyWriter::new(ctx, Box::new(dev))
        },
        &[T::System, T::KeyOutput],
    )?;

    supervisor.add_actor(
        "KeyEventPipeline",
        |ctx| {
            let processors: Vec<Box<dyn Processor + Send + Sync>> = vec![
                Box::new(KeyEventProcessor::default()),
                Box::new(SystemShortcutProcessor::new(state.clone())),
            ];
            Pipeline::new(ctx, processors)
        },
        &[T::System, T::KeyInput],
    )?;

    supervisor.add_actor(
        "IPCServer",
        |ctx| IPCServer::new(ctx, state.clone()),
        &[T::System, T::Stats, T::Monitoring],
    )?;

    if config.sleep_script.is_some() && config.awake_script.is_some() {
        supervisor.add_actor(
            "PowerManager",
            |ctx| PowerManager::new(ctx, state.clone()),
            &[T::System, T::KeyInput],
        )?;
    }

    let raw_enabled = config.keyboard_info().is_some_and(|group| {
        group.raw_hid_enabled && group.vendor_id.is_some() && group.product_id.is_some()
    });
    if raw_enabled {
        // let alias = match config.keyboard {
        //     InputConfig::Use(ref keyb) => keyb.clone(),
        //     ref k => Err(CharonError::QMKError(format!(
        //         "{k:?} - insufficient or wrong configuration to enable QMK actor"
        //     )))?,
        // };
        let device = QmkAsyncHidDevice::async_new(&config).await;
        supervisor.add_actor(
            "QMK",
            |ctx| QMK::new(ctx, state.clone(), Box::new(device)),
            &[T::System],
        )?;
    }

    supervisor.add_actor(
        "Typist",
        |ctx| Typist::new(ctx, state.clone(), keymap),
        &[T::System, T::TextInput],
    )?;

    if config.enable_telemetry {
        supervisor.add_actor(
            "Telemetry",
            Telemetry::new,
            &[T::System, T::Telemetry, T::KeyInput, T::Stats],
        )?;
    }

    supervisor.add_actor(
        "TypingStats",
        |ctx| TypingStats::new(ctx, state.clone()),
        &[T::System, T::KeyInput],
    )?;

    let mut sigterm = unix::signal(unix::SignalKind::terminate())?;

    tokio::select! {
        _ = supervisor.run() => {},
        // _ = daemon.run() => {},
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
            // daemon.stop().await;
        },
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down...");
            // daemon.stop().await;
        }
    }

    supervisor.stop().await?;

    info!("Charon says goodbye. Hades is waiting...");
    Ok(())
}

fn get_config() -> eyre::Result<CharonConfig> {
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
