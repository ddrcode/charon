use std::collections::HashMap;

use charon_lib::event::{DomainEvent, Event};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::domain::{Actor, ActorState};

pub struct Telemetry {
    state: ActorState,
    events: HashMap<Uuid, u128>,
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
            DomainEvent::KeySent(time) => {
                self.events.insert(event.source_event_id.unwrap(), *time);
            }
            DomainEvent::ReportConsumed(_) => {
                self.events.remove(&event.source_event_id.unwrap());
            }
            DomainEvent::ReportSent(time) => {
                let timestamp = self.events.remove(&event.source_event_id.unwrap()).unwrap();
                if let Some(diff) = time.checked_sub(timestamp) {
                    println!("Key to report time: {}", diff);
                }
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
