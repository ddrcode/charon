pub mod actor;
pub mod broker;
pub mod domain;
pub mod error;

use actor::{
    key_scanner::{self, spawn_key_scanner},
    passthrough::{self, spawn_pass_through},
};
use anyhow;
use broker::EventBroker;
use domain::Event;
use tokio::{self, signal::unix, sync::mpsc, task::JoinHandle};
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use crate::domain::DomainEvent;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let mut tasks: Vec<JoinHandle<()>> = Vec::new();

    let (event_tx, broker_rx) = mpsc::channel::<Event>(128);
    let mut broker = EventBroker::new(broker_rx);

    let (scan_tx, scan_rx) = mpsc::channel::<Event>(128);
    broker.add_subscriber(scan_tx, key_scanner::filter);
    tasks.push(spawn_key_scanner(event_tx.clone(), scan_rx).await);

    let (pt_tx, pt_rx) = mpsc::channel::<Event>(128);
    broker.add_subscriber(pt_tx, passthrough::filter);
    tasks.push(spawn_pass_through(event_tx.clone(), pt_rx));

    let mut sigterm = unix::signal(unix::SignalKind::terminate())?;

    tokio::select! {
        _ = broker.run() => {},
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down gracefully...");
            exit(&mut broker).await;
        },
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down");
            exit(&mut broker).await
        }
    }

    for handle in tasks {
        let _ = handle.await;
    }

    info!("Charon says goodbye. Hades is waiting...");
    Ok(())
}

async fn exit(broker: &mut EventBroker) {
    let event = Event::new("broker", DomainEvent::Exit);
    broker.broadcast(&event, true).await;
}
