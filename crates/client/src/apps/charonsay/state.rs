use std::time::Duration;

use crate::{apps::charonsay::WisdomCategory, config::AppConfig};

use super::ascii_art::LOGO;

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub art: &'static str,
    pub wisdom: String,
    pub title: &'static str,
    pub time_to_next: Duration,
    pub time_to_idle: Duration,
    pub view: WisdomCategory,
    pub wpm: u16,
    pub total_keys: u64,
}

impl State {
    pub fn from_config(config: &AppConfig) -> Self {
        let mut state = State::default();
        state.time_to_next = config.splash_duration;
        state.time_to_idle = config.idle_time;
        state
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            art: LOGO,
            wisdom: "Charon is rowing...\n\nPress the <[magic key]> to take control".into(),
            title: "".into(),
            time_to_next: Duration::from_secs(180),
            time_to_idle: Duration::from_secs(300),
            view: WisdomCategory::default(),
            wpm: 0,
            total_keys: 0,
        }
    }
}
