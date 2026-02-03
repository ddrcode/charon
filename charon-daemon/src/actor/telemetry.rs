// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{domain::CharonEvent, port::Metrics};
use lru_time_cache::LruCache;
use maiko::{Envelope, StepAction};
use std::time::Duration;
use tracing::warn;

pub struct Telemetry<M: Metrics> {
    events: LruCache<u128, u64>,
    metrics: M,
    push_interval: Duration,
}

impl<M: Metrics> Telemetry<M> {
    pub fn new(metrics: M) -> Self {
        Self {
            events: LruCache::with_expiry_duration_and_capacity(Duration::from_secs(10), 1024),
            metrics,
            push_interval: Duration::from_secs(15),
        }
    }
}

impl<M: Metrics> maiko::Actor for Telemetry<M> {
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
            _ => {}
        }
        Ok(())
    }

    async fn step(&mut self) -> maiko::Result<StepAction> {
        if let Err(e) = self.metrics.flush().await {
            tracing::error!("Sending telemetry failed: {e}");
        }
        Ok(StepAction::Backoff(self.push_interval))
    }
}
