use charon_lib::domain::{DomainEvent, Event};
use tracing::{info, warn};

use crate::domain::ActorState;

#[async_trait::async_trait]
pub trait Actor {
    fn id(&self) -> &'static str {
        self.state().id
    }

    fn state(&self) -> &ActorState;

    fn state_mut(&mut self) -> &mut ActorState;

    async fn send(&mut self, payload: DomainEvent) {
        let event = Event::new(self.id(), payload);
        if let Err(_) = self.state().sender.send(event).await {
            warn!("Channel closed while sending event, quitting");
            self.stop().await;
        }
    }

    async fn recv(&mut self) -> Option<Event> {
        let maybe_event = self.state_mut().receiver.recv().await;
        if maybe_event.is_none() {
            warn!("Channel closed while receiving event, quitting");
        }
        maybe_event
    }

    async fn run(&mut self) {
        info!("Starting actor: {}", self.id());
        self.init().await;
        while self.state().alive {
            self.tick().await;
        }
        self.shutdown().await;
    }

    async fn tick(&mut self);

    async fn stop(&mut self) {
        self.state_mut().alive = false;
        info!("Actor: {} is stopping", self.id());
    }

    async fn init(&mut self) {}
    async fn shutdown(&mut self) {}
}
