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
        // self.state().id.clone()
        "id".into()
    }

    fn state(&self) -> &ActorState;

    fn state_mut(&mut self) -> &mut ActorState;

    async fn send(&mut self, payload: DomainEvent) {
        let event = Event::new(self.id(), payload);
        self.send_raw(event).await;
    }

    async fn send_raw(&mut self, event: Event) {
        // if let Err(_) = self.state().sender.send(event).await {
        //     warn!(
        //         "Channel closed for {} while sending event, quitting",
        //         self.id()
        //     );
        //     self.stop().await;
        // }
    }

    async fn process(&mut self, payload: DomainEvent) {
        self.process_raw(Event::new(self.id(), payload)).await;
    }

    async fn process_raw(&mut self, _event: Event) {
        // let mut events = vec![event];
        //
        // for proc in &mut self.state_mut().iter_processors() {
        //     let mut next_events = Vec::new();
        //     for event in events {
        //         let mut out = proc.process(event).await;
        //         next_events.append(&mut out);
        //     }
        //     events = next_events;
        // }
        //
        // for mut ev in events {
        //     ev.sender = self.id();
        //     self.send_raw(ev).await;
        // }
    }

    async fn recv(&mut self) -> Option<Event> {
        // if self.state().alive == false {
        //     return None;
        // }
        // let maybe_event = self.state_mut().receiver.recv().await;
        // if maybe_event.is_none() {
        //     warn!(
        //         "Channel closed for {} while receiving event, quitting",
        //         self.id()
        //     );
        //     self.stop().await;
        // }
        // maybe_event
        None
    }

    async fn run(&mut self) {
        info!("Starting actor: {}", self.id());
        self.init().await;

        // while self.state().alive {
        //     self.tick().await;
        // }

        self.shutdown().await;
    }

    async fn tick(&mut self);

    async fn stop(&mut self) {
        let state = self.state_mut();
        // state.alive = false;
        // state.receiver.close();
        info!("Actor: {} is stopping", self.id());
    }

    async fn init(&mut self) {}
    async fn shutdown(&mut self) {}

    fn spawn(state: ActorState, init: Self::Init) -> Result<JoinHandle<()>, CharonError>;
}
