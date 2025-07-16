use std::time::Duration;
use tokio::time::Instant;

pub struct DynamicInterval {
    next_refresh: Option<Instant>,
    interval: Duration,
}

impl DynamicInterval {
    pub fn new(interval: Duration) -> Self {
        Self {
            next_refresh: Some(Instant::now() + interval),
            interval,
        }
    }

    pub fn reset(&mut self) {
        self.next_refresh = Some(Instant::now() + self.interval);
    }

    pub fn reset_with(&mut self, new_interval: Duration) {
        self.interval = new_interval;
        self.next_refresh = Some(Instant::now() + new_interval);
    }

    /// Completely stop the timer from ever firing
    pub fn stop(&mut self) {
        self.next_refresh = None;
    }

    pub fn is_active(&self) -> bool {
        self.next_refresh.is_some()
    }

    pub async fn sleep_until(&self) {
        if let Some(instant) = self.next_refresh {
            tokio::time::sleep_until(instant).await;
        } else {
            futures::future::pending::<()>().await;
        }
    }

    pub fn remaining(&self) -> Option<Duration> {
        self.next_refresh
            .map(|t| t.saturating_duration_since(Instant::now()))
    }
}
