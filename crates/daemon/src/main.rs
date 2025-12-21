pub mod actor;
pub mod adapter;
pub mod broker;
pub mod config;
pub mod daemon;
pub mod domain;
pub mod error;
pub mod port;
pub mod processor;
pub mod util;

use charon_lib::event::{Event, Mode, Topic as T};
use maiko::Supervisor;
use std::{fs::read_to_string, path::PathBuf, sync::Arc};
use tokio::{
    self,
    io::unix::AsyncFd,
    signal::unix,
    sync::{RwLock, mpsc},
};
use tracing::{debug, info, warn};
use tracing_subscriber::FmtSubscriber;

use crate::{
    actor::{KeyScanner, KeyWriter, Pipeline},
    adapter::{EventDeviceUnix, HIDDeviceUnix},
    config::CharonConfig,
    domain::{ActorState, ProcessorState, traits::Processor},
    error::CharonError,
    processor::{KeyEventProcessor, SystemShortcutProcessor},
    util::evdev::find_input_device,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init_logging();

    let config = get_config().expect("Failed loading config file");
    // info!("Loading keymap");
    // let keymap = KeymapLoaderYaml::new(&config.keymaps_dir)
    //     .load_keymap(&config.host_keymap)
    //     .await?;

    let mode = Arc::new(RwLock::new(Mode::PassThrough));
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

                let (tx, rx) = mpsc::channel::<Event>(128);
                let state = ActorState::new(
                    format!("KeyScanner-{name}").into(),
                    mode.clone(),
                    tx,
                    rx,
                    config,
                    Vec::new(),
                );

                KeyScanner::new(ctx, state, Box::new(input), name)
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
            let state = ProcessorState::new(mode, config.clone());
            let processors: Vec<Box<dyn Processor + Send + Sync>> = vec![
                Box::new(KeyEventProcessor::default()),
                Box::new(SystemShortcutProcessor::new(state)),
            ];
            Pipeline::new(ctx, processors)
        },
        &[T::System, T::KeyInput],
    )?;

    // let mut daemon = Daemon::new();
    // daemon
    //     .with_config(config.clone())
    //     .add_scanners(&[T::System])
    //     .add_actor::<KeyWriter>(&[T::System, T::KeyOutput])
    //     .add_actor_with_init::<Typist>(keymap, &[T::System, T::TextInput])
    //     .add_actor::<TypingStats>(&[T::System, T::KeyInput])
    //     .add_actor::<IPCServer>(&[T::System, T::Stats, T::Monitoring])
    //     .add_pipeline(
    //         "PassThroughPipeline",
    //         &[T::System, T::KeyInput],
    //         &[KeyEventProcessor::factory, SystemShortcutProcessor::factory],
    //     );
    // .add_actor_conditionally::<PowerManager>(
    //     config.sleep_script.is_some() && config.awake_script.is_some(),
    //     &[T::System, T::KeyInput],
    // )
    // .add_actor_conditionally::<Telemetry>(
    //     config.enable_telemetry,
    //     &[T::System, T::Telemetry, T::KeyInput, T::Stats],
    // )
    // .add_actor_conditionally::<QMK>(true, &[T::System]);

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

    // daemon.shutdown().await;

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
