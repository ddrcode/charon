use std::{path::Path, time::Duration};

use charon_lib::{
    event::{DomainEvent, Event},
    stats::CurrentStats,
    util::time::{is_today, next_midnight_instant},
};
use tokio::{select, task::JoinHandle};
use tracing::{error, info};

use super::WPMCounter;
use crate::{
    domain::{ActorState, traits::Actor},
    error::CharonError,
};

pub struct TypingStats {
    state: ActorState,
    wpm: WPMCounter,
    total_count: u64,
    today_count: u64,
}

impl TypingStats {
    pub fn new(state: ActorState) -> Self {
        let wpm = WPMCounter::new(
            Duration::from_secs(state.config().stats_wpm_slot_duration),
            state.config().stats_wpm_slot_count,
        );
        Self {
            state,
            wpm,
            total_count: 0,
            today_count: 0,
        }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::Exit => self.stop().await,
            DomainEvent::KeyPress(key, _) => {
                self.wpm.register_key(key);
                self.total_count += 1;
                self.today_count += 1;
            }
            _ => {}
        }
    }

    async fn load_stats(&self, file: &Path) -> std::io::Result<CurrentStats> {
        let data = tokio::fs::read_to_string(file).await?;
        let mut stats = serde_json::from_str::<CurrentStats>(&data)?;
        let today_count = stats.today;
        stats.today = 0;
        if let Ok(meta) = tokio::fs::metadata(file).await {
            if let Ok(time) = meta.modified() {
                if is_today(time) {
                    stats.today = today_count;
                }
            }
        }
        Ok(stats)
    }

    async fn write_stats(&self, file: &Path, stats: CurrentStats) {
        if let Ok(txt) = serde_json::to_string(&stats) {
            if let Err(err) = tokio::fs::write(file, txt).await {
                error!("Couldn't write stats file: {err}");
            }
        }
    }

    fn stats(&self) -> CurrentStats {
        CurrentStats::new(
            self.today_count,
            self.total_count,
            self.wpm.wpm(),
            self.wpm.max_wpm(),
        )
    }
}

#[async_trait::async_trait]
impl Actor for TypingStats {
    type Init = ();

    fn name() -> &'static str {
        "TypingStats"
    }

    fn spawn(state: ActorState, (): ()) -> Result<JoinHandle<()>, CharonError> {
        let mut stats = TypingStats::new(state);
        Ok(tokio::spawn(async move { stats.run().await }))
    }

    async fn init(&mut self) {
        match self.load_stats(&self.state.config().stats_file).await {
            Ok(stats) => {
                self.total_count = stats.total;
                self.today_count = stats.today;
                self.wpm.set_wpm_max(stats.max_wpm);
            }
            Err(err) => {
                error!("Couldn't load stats file: {err}");
            }
        }
    }

    async fn run(&mut self) {
        info!("Starting actor: {}", self.id());
        self.init().await;

        let mut wpm_interval = tokio::time::interval(self.wpm.period());
        let mut save_interval =
            tokio::time::interval(Duration::from_secs(self.state.config().stats_save_interval));

        while self.state().alive {
            select! {
                Some(event) = self.recv() => {
                    self.handle_event(&event).await;
                }
                _ = wpm_interval.tick() => {
                    self.wpm.next();
                    self.send(DomainEvent::CurrentStats(self.stats())).await;
                }
                _ = save_interval.tick() => {
                    self.write_stats(&self.state.config().stats_file, self.stats()).await;
                }
                _ = tokio::time::sleep_until(next_midnight_instant()) => {
                    self.today_count = 0;
                }
            }
        }

        self.shutdown().await;
    }

    async fn shutdown(&mut self) {
        self.write_stats(&self.state.config().stats_file, self.stats())
            .await;
    }

    async fn tick(&mut self) {}

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }
}
