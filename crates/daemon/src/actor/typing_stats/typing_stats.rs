use std::{path::Path, time::Duration};

use charon_lib::{
    event::CharonEvent,
    stats::CurrentStats,
    util::time::{is_today, next_midnight_instant},
};
use maiko::{Context, Meta};
use tokio::select;
use tracing::error;

use super::WPMCounter;
use crate::domain::ActorState;

pub struct TypingStats {
    ctx: Context<CharonEvent>,
    state: ActorState,
    wpm: WPMCounter,
    total_count: u64,
    today_count: u64,
    wpm_interval: tokio::time::Interval,
    save_interval: tokio::time::Interval,
}

impl TypingStats {
    pub fn new(ctx: Context<CharonEvent>, state: ActorState) -> Self {
        let wpm = WPMCounter::new(
            Duration::from_secs(state.config().stats_wpm_slot_duration),
            state.config().stats_wpm_slot_count,
        );
        Self {
            ctx,
            total_count: 0,
            today_count: 0,
            wpm_interval: tokio::time::interval(wpm.period()),
            save_interval: tokio::time::interval(Duration::from_secs(
                state.config().stats_save_interval,
            )),
            wpm,
            state,
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

impl maiko::Actor for TypingStats {
    type Event = CharonEvent;

    async fn on_start(&mut self) -> maiko::Result {
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
        Ok(())
    }

    async fn handle_event(&mut self, event: &Self::Event) -> maiko::Result {
        match event {
            CharonEvent::Exit => self.ctx.stop(),
            CharonEvent::KeyPress(key, _) => {
                self.wpm.register_key(key);
                self.total_count += 1;
                self.today_count += 1;
            }
            _ => {}
        }
        Ok(())
    }

    async fn tick(&mut self) -> maiko::Result {
        select! {
            _ = self.wpm_interval.tick() => {
                self.wpm.next();
                self.ctx.send(CharonEvent::CurrentStats(self.stats())).await?;
            }
            _ = self.save_interval.tick() => {
                self.write_stats(&self.state.config().stats_file, self.stats()).await;
            }
            _ = tokio::time::sleep_until(next_midnight_instant()) => {
                self.today_count = 0;
            }
        }
        Ok(())
    }

    async fn on_shutdown(&mut self) -> maiko::Result {
        self.write_stats(&self.state.config().stats_file, self.stats())
            .await;
        Ok(())
    }
}
