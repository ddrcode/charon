use std::borrow::Cow;

use charon_lib::event::{DomainEvent, Event};
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::{domain::ActorState, error::CharonError};

#[async_trait::async_trait]
pub trait Actor {
    type Init: Send + Sync + 'static;

    fn name() -> &'static str;

    fn id(&self) -> Cow<'static, str> {
        self.state().id.clone()
    }

    fn state(&self) -> &ActorState;

    fn state_mut(&mut self) -> &mut ActorState;

    async fn send(&mut self, payload: DomainEvent) {
        let event = Event::new(self.id(), payload);
        self.send_raw(event).await;
    }

    async fn send_raw(&mut self, event: Event) {
        if let Err(_) = self.state().sender.send(event).await {
            warn!(
                "Channel closed for {} while sending event, quitting",
                self.id()
            );
            self.stop().await;
        }
    }

    async fn recv(&mut self) -> Option<Event> {
        if self.state().alive == false {
            return None;
        }
        let maybe_event = self.state_mut().receiver.recv().await;
        if maybe_event.is_none() {
            warn!(
                "Channel closed for {} while receiving event, quitting",
                self.id()
            );
            self.stop().await;
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
        let state = self.state_mut();
        state.alive = false;
        state.receiver.close();
        info!("Actor: {} is stopping", self.id());
    }

    async fn init(&mut self) {}
    async fn shutdown(&mut self) {}

    fn spawn(state: ActorState, init: Self::Init) -> Result<JoinHandle<()>, CharonError>;
}
