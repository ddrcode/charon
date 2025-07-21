use std::time::Duration;

use charon_lib::{
    event::{DomainEvent, Event},
    stats::CurrentStats,
};
use tokio::{select, task::JoinHandle};
use tracing::info;

use super::WPMCounter;
use crate::domain::{ActorState, traits::Actor};

pub struct TypingStats {
    state: ActorState,
    wpm: WPMCounter,
    total_count: u64,
}

impl TypingStats {
    pub fn new(state: ActorState) -> Self {
        Self {
            state,
            wpm: WPMCounter::new(),
            total_count: 0,
        }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::Exit => self.stop().await,
            DomainEvent::KeyPress(key, _) => {
                self.wpm.register_key(key);
                self.total_count += 1;
            }
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

    async fn run(&mut self) {
        info!("Starting actor: {}", self.id());
        self.init().await;

        let mut interval = tokio::time::interval(Duration::from_secs(5));

        while self.state().alive {
            select! {
                Some(event) = self.recv() => {
                    self.handle_event(&event).await;
                }
                _ = interval.tick() => {
                    self.wpm.next();
                    self.send(DomainEvent::CurrentStats(CurrentStats::new(self.total_count, self.wpm.wpm()))).await;
                }
            }
        }

        self.shutdown().await;
    }

    async fn tick(&mut self) {}

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }
}
