use tokio::{
    sync::mpsc::{self, Sender},
    task::JoinHandle,
};

use crate::{
    actor::{
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
        let (scan_tx, scan_rx) = mpsc::channel::<Event>(128);
        self.broker.add_subscriber(scan_tx, key_scanner::filter);
        let task = spawn_key_scanner(self.event_tx.clone(), scan_rx).await;
        self.tasks.push(task);

        let (pt_tx, pt_rx) = mpsc::channel::<Event>(128);
        self.broker.add_subscriber(pt_tx, passthrough::filter);
        let task = spawn_pass_through(self.event_tx.clone(), pt_rx);
        self.tasks.push(task);

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
}
