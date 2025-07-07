use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    actor::{
        ipc_server::{self, spawn_ipc_server},
        key_scanner::{self, spawn_key_scanner},
        passthrough::{self, spawn_pass_through},
    },
    broker::EventBroker,
    domain::{DomainEvent, Event},
};

pub struct Daemon {
    tasks: Vec<JoinHandle<()>>,
    broker: EventBroker,
    event_tx: Sender<Event>,
}

impl Daemon {
    pub fn new() -> Self {
        let (event_tx, broker_rx) = mpsc::channel::<Event>(128);
        Self {
            tasks: Vec::new(),
            broker: EventBroker::new(broker_rx),
            event_tx,
        }
    }

    pub async fn start(&mut self) {
        self.add_actor(spawn_key_scanner, key_scanner::filter)
            .add_actor(spawn_pass_through, passthrough::filter)
            .add_actor(spawn_ipc_server, ipc_server::filter);

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
        spawn_fn: fn(Sender<Event>, Receiver<Event>) -> JoinHandle<()>,
        filter_fn: fn(&Event) -> bool,
    ) -> &mut Self {
        let (pt_tx, pt_rx) = mpsc::channel::<Event>(128);
        self.broker.add_subscriber(pt_tx, filter_fn);
        let task = spawn_fn(self.event_tx.clone(), pt_rx);
        self.tasks.push(task);
        self
    }
}
