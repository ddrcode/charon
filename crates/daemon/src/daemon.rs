use std::sync::Arc;

use charon_lib::domain::{DomainEvent, Event, Mode};
use tokio::{
    sync::{
        RwLock,
        mpsc::{self, Sender},
    },
    task::JoinHandle,
};
use tracing::info;

use crate::{
    actor::{
        ipc_server::{self, spawn_ipc_server},
        key_scanner::{self, spawn_key_scanner},
        passthrough::{self, spawn_pass_through},
    },
    broker::EventBroker,
    domain::ActorState,
};

pub struct Daemon {
    tasks: Vec<JoinHandle<()>>,
    broker: EventBroker,
    event_tx: Sender<Event>,
    mode: Arc<RwLock<Mode>>,
}

impl Daemon {
    pub fn new() -> Self {
        let (event_tx, broker_rx) = mpsc::channel::<Event>(128);
        Self {
            tasks: Vec::new(),
            broker: EventBroker::new(broker_rx),
            event_tx,
            mode: Arc::new(RwLock::new(Mode::PassThrough)),
        }
    }

    pub async fn start(&mut self) {
        self.add_actor("KeyScanner", spawn_key_scanner, key_scanner::filter)
            .add_actor("PassThrough", spawn_pass_through, passthrough::filter)
            .add_actor("IPCServer", spawn_ipc_server, ipc_server::filter);

        info!("Charon is ready...");
        self.broker.run().await;
    }

    pub async fn stop(&mut self) {
        let event = Event::new("broker", DomainEvent::Exit);
        self.broker.broadcast(&event, true).await;
    }

    pub async fn shutdown(&mut self) {
        for handle in self.tasks.drain(..) {
            handle.await.unwrap();
        }
    }

    fn add_actor(
        &mut self,
        name: &'static str,
        spawn_fn: fn(ActorState) -> JoinHandle<()>,
        filter_fn: fn(&Event) -> bool,
    ) -> &mut Self {
        let (pt_tx, pt_rx) = mpsc::channel::<Event>(128);
        self.broker.add_subscriber(pt_tx, filter_fn);
        let state = ActorState::new(name, self.mode.clone(), self.event_tx.clone(), pt_rx);
        let task = spawn_fn(state);
        self.tasks.push(task);
        self
    }
}
