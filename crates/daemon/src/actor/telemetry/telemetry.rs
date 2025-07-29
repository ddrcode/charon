use charon_lib::event::{DomainEvent, Event};
use lru_time_cache::LruCache;
use std::time::Duration;
use tokio::{select, task::JoinHandle};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    actor::telemetry::MetricsManager,
    domain::{ActorState, traits::Actor},
    error::CharonError,
};

pub struct Telemetry {
    state: ActorState,
    events: LruCache<Uuid, u64>,
    metrics: MetricsManager,
}

impl Telemetry {
    pub fn new(state: ActorState, metrics: MetricsManager) -> Self {
        Self {
            state,
            events: LruCache::with_expiry_duration_and_capacity(Duration::from_secs(10), 1024),
            metrics,
        }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::KeyPress(key, keyboard) => {
                self.events.insert(event.id, event.timestamp);
                self.metrics.register_key_event(key, keyboard);
            }
            DomainEvent::KeyRelease(..) => {
                self.events.insert(event.id, event.timestamp);
            }
            DomainEvent::ReportSent() => {
                if let Some(ref source_id) = event.source_event_id {
                    if let Some(timestamp) = self.events.remove(source_id) {
                        if let Some(diff) = event.timestamp.checked_sub(timestamp) {
                            self.metrics.register_key_to_report_time(diff);
                        }
                    }
                } else {
                    warn!(
                        "Missing source_event_id for ReportSent event, id: {}",
                        event.id
                    );
                }
            }
            DomainEvent::CurrentStats(stats) => {
                self.metrics.register_wpm(stats.wpm);
            }
            DomainEvent::Exit => self.stop().await,
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl Actor for Telemetry {
    type Init = ();

    fn name() -> &'static str {
        "Telemetry"
    }

    fn spawn(state: ActorState, (): ()) -> Result<JoinHandle<()>, CharonError> {
        let metrics = MetricsManager::new()?;
        let mut telemetry = Telemetry::new(state, metrics);
        Ok(tokio::spawn(async move { telemetry.run().await }))
    }

    async fn run(&mut self) {
        info!("Starting actor: {}", self.id());
        self.init().await;

        let mut push_interval = tokio::time::interval(Duration::from_secs(15));

        while self.state().alive {
            select! {
                Some(event) = self.recv() => {
                    self.handle_event(&event).await;
                }
                _ = push_interval.tick() => {
                    self.metrics.push().await;
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

    async fn init(&mut self) {
        self.metrics.start_server().await;
    }

    async fn shutdown(&mut self) {
        self.metrics.stop_server().await;
    }
}
