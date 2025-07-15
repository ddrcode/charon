use std::collections::HashMap;

use charon_lib::event::{DomainEvent, Event};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::domain::{Actor, ActorState};

pub struct Telemetry {
    state: ActorState,
    events: HashMap<Uuid, u64>,
}

impl Telemetry {
    pub fn new(state: ActorState) -> Self {
        Self {
            state,
            events: HashMap::with_capacity(1024),
        }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::KeyPress(_) | DomainEvent::KeyRelease(_) => {
                self.events.insert(event.id, event.timestamp);
            }
            DomainEvent::ReportConsumed(_) => {
                self.events.remove(&event.source_event_id.unwrap());
            }
            DomainEvent::ReportSent(_) => {
                let timestamp = self.events.remove(&event.source_event_id.unwrap()).unwrap();
                println!("Key to report time: {}", event.timestamp - timestamp);
            }
            DomainEvent::Exit => self.stop().await,
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl Actor for Telemetry {
    fn spawn(state: ActorState) -> JoinHandle<()> {
        let mut telemetry = Telemetry::new(state);
        tokio::spawn(async move { telemetry.run().await })
    }

    async fn tick(&mut self) {
        if let Some(event) = self.recv().await {
            self.handle_event(&event).await;
        }
    }

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }
}
