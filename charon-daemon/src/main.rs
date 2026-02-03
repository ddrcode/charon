// SPDX-License-Identifier: GPL-3.0-or-later
pub mod actor;
pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod port;
pub mod processor;
pub mod util;

use crate::{
    adapter::PrometheusMetrics,
    domain::{Mode, Topic as T},
};
use maiko::Supervisor;
use std::sync::Arc;
use tokio::{self, io::unix::AsyncFd, signal::unix};
use tracing_subscriber::FmtSubscriber;

use crate::{
    actor::{
        KeyScanner, KeyWriter, Pipeline, PowerManager, QMK, Telemetry, TypingStats, Typist,
        ipc_bridge::IPCServer,
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

    let config = Arc::new(CharonConfig::from_file().expect("Failed loading config file"));
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
                KeyScanner::new(ctx, state.clone(), input, name)
            },
            [T::System],
        )?;
    }

    supervisor.add_actor(
        "KeyWriter",
        |ctx| {
            let dev_path = config.hid_keyboard.clone();
            let dev = HIDDeviceUnix::new(&dev_path);
            KeyWriter::new(ctx, dev)
        },
        [T::System, T::KeyOutput],
    )?;

    supervisor.add_actor(
        "KeyEventPipeline",
        |ctx| {
            let processors: Vec<Box<dyn Processor + Send + Sync>> = vec![
                Box::new(KeyEventProcessor::default()),
                Box::new(SystemShortcutProcessor::new(ctx.clone(), state.clone())),
            ];
            Pipeline::new(ctx, processors)
        },
        [T::System, T::KeyInput],
    )?;

    supervisor.add_actor(
        "IPCServer",
        |ctx| IPCServer::new(ctx, state.clone()),
        [T::System, T::Stats, T::Monitoring],
    )?;

    if config.sleep_script.is_some() && config.awake_script.is_some() {
        supervisor.add_actor(
            "PowerManager",
            |ctx| PowerManager::new(ctx, state.clone()),
            [T::System, T::KeyInput],
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
            |ctx| QMK::new(ctx, state.clone(), device),
            [T::System],
        )?;
    }

    supervisor.add_actor(
        "Typist",
        |ctx| Typist::new(ctx, state.clone(), keymap),
        &[T::System, T::TextInput],
    )?;

    if config.enable_telemetry {
        let prometheus = PrometheusMetrics::new()?;
        supervisor.add_actor(
            "Telemetry",
            |_ctx| Telemetry::new(prometheus),
            [T::System, T::Telemetry, T::KeyInput, T::Stats],
        )?;
    }

    supervisor.add_actor(
        "TypingStats",
        |ctx| TypingStats::new(ctx, state.clone()),
        [T::System, T::KeyInput],
    )?;

    let mut sigterm = unix::signal(unix::SignalKind::terminate())?;

    tokio::select! {
        _ = supervisor.run() => {},
        // _ = daemon.run() => {},
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received Ctrl+C, shutting down...");
            // daemon.stop().await;
        },
        _ = sigterm.recv() => {
            tracing::info!("Received SIGTERM, shutting down...");
            // daemon.stop().await;
        }
    }

    supervisor.stop().await?;

    tracing::info!("Charon says goodbye. Hades is waiting...");
    Ok(())
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
