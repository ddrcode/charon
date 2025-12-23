use charon_lib::event::DomainEvent;
use lru_time_cache::LruCache;
use maiko::{Context, Meta};
use std::time::Duration;
use tracing::warn;

use crate::actor::telemetry::MetricsManager;

pub struct Telemetry {
    ctx: Context<DomainEvent>,
    events: LruCache<u128, u64>,
    metrics: MetricsManager,
    push_interval: tokio::time::Interval,
}

impl Telemetry {
    pub fn new(ctx: Context<DomainEvent>) -> Self {
        Self {
            ctx,
            events: LruCache::with_expiry_duration_and_capacity(Duration::from_secs(10), 1024),
            metrics: MetricsManager::new().expect("Prometheus metrics should initialize correctly"),
            push_interval: tokio::time::interval(Duration::from_secs(15)),
        }
    }
}

impl maiko::Actor for Telemetry {
    type Event = DomainEvent;

    async fn handle(&mut self, event: &DomainEvent, meta: &Meta) -> maiko::Result {
        match event {
            DomainEvent::KeyPress(key, keyboard) => {
                self.events.insert(meta.id(), meta.timestamp());
                self.metrics.register_key_event(key, keyboard);
            }
            DomainEvent::KeyRelease(..) => {
                self.events.insert(meta.id(), meta.timestamp());
            }
            DomainEvent::ReportSent => {
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
            DomainEvent::CurrentStats(stats) => {
                self.metrics.register_wpm(stats.wpm);
            }
            DomainEvent::Exit => self.ctx.stop(),
            _ => {}
        }
        Ok(())
    }

    async fn tick(&mut self) -> maiko::Result {
        self.push_interval.tick().await;
        self.metrics.push().await;
        Ok(())
    }
}
