use charon_lib::event::{DomainEvent, Event};
use tokio::task::JoinHandle;

use crate::domain::{Actor, ActorState};

pub struct TypingStats {
    state: ActorState,
}

impl TypingStats {
    pub fn new(state: ActorState) -> Self {
        Self { state }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::Exit => self.stop().await,
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl Actor for TypingStats {
    type Init = ();

    fn name() -> &'static str {
        "TypingStats"
    }

    fn spawn(state: ActorState, (): ()) -> JoinHandle<()> {
        let mut stats = TypingStats::new(state);
        tokio::spawn(async move { stats.run().await })
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
