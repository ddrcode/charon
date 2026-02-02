use std::sync::{Arc, Mutex};

use evdev::KeyCode;

use crate::{error::CharonError, port::Metrics};

pub struct MetricsState {
    wpm_counter: usize,
    last_wpm: u16,

    key_events_counter: usize,
    last_key_event: KeyCode,

    key_to_report_time_counter: usize,
    last_key_to_report_time: u64,
}

pub struct MetricsMock {
    state: Arc<Mutex<MetricsState>>,
}

impl Metrics for MetricsMock {
    fn register_key_event(&self, key: &evdev::KeyCode, _keyboard: &str) {
        if let Ok(mut state) = self.state.lock() {
            state.key_events_counter += 1;
            state.last_key_event = *key;
        }
    }

    fn register_key_to_report_time(&self, time: u64) {
        if let Ok(mut state) = self.state.lock() {
            state.key_to_report_time_counter += 1;
            state.last_key_to_report_time = time;
        }
    }

    fn register_wpm(&self, wpm: u16) {
        if let Ok(mut state) = self.state.lock() {
            state.wpm_counter += 1;
            state.last_wpm = wpm;
        }
    }

    async fn flush(&mut self) -> Result<(), CharonError> {
        Ok(())
    }
}
