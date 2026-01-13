use charon_lib::event::CharonEvent;
use lru_time_cache::LruCache;
use maiko::{Context, Envelope, StepAction};
use std::time::Duration;
use tracing::warn;

use crate::actor::telemetry::MetricsManager;

pub struct Telemetry {
    ctx: Context<CharonEvent>,
    events: LruCache<u128, u64>,
    metrics: MetricsManager,
    push_interval: Duration,
}

impl Telemetry {
    pub fn new(ctx: Context<CharonEvent>) -> Self {
        Self {
            ctx,
            events: LruCache::with_expiry_duration_and_capacity(Duration::from_secs(10), 1024),
            metrics: MetricsManager::new().expect("Prometheus metrics should initialize correctly"),
            push_interval: Duration::from_secs(15),
        }
    }
}

impl maiko::Actor for Telemetry {
    type Event = CharonEvent;

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> maiko::Result {
        let meta = envelope.meta();
        match envelope.event() {
            CharonEvent::KeyPress(key, keyboard) => {
                self.events.insert(meta.id(), meta.timestamp());
                self.metrics.register_key_event(key, keyboard);
            }
            CharonEvent::KeyRelease(..) => {
                self.events.insert(meta.id(), meta.timestamp());
            }
            CharonEvent::ReportSent => {
                if let Some(ref source_id) = meta.correlation_id() {
                    if let Some(timestamp) = self.events.remove(source_id) {
                        if let Some(diff) = meta.timestamp().checked_sub(timestamp) {
                            self.metrics.register_key_to_report_time(diff);
                        }
                    }
                } else {
                    warn!(
                        "Missing source_event_id for ReportSent event, id: {}",
                        meta.id()
                    );
                }
            }
            CharonEvent::CurrentStats(stats) => {
                self.metrics.register_wpm(stats.wpm);
            }
            CharonEvent::Exit => self.ctx.stop(),
            _ => {}
        }
        Ok(())
    }

    async fn step(&mut self) -> maiko::Result<StepAction> {
        self.metrics.push().await;
        Ok(StepAction::Backoff(self.push_interval))
    }
}
