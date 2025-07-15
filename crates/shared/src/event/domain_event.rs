use evdev::KeyCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Mode, Topic};
use crate::stats::CurrentStats;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DomainEvent {
    KeyPress(KeyCode),
    KeyRelease(KeyCode),
    HidReport([u8; 8]),
    SendText(String),
    SendFile(String, bool),
    TextSent,
    CurrentStats(CurrentStats),
    ModeChange(Mode),
    Exit,

    // telemetry events
    KeySent(u128),
    ReportSent(u128),
    ReportConsumed(u128),
}

impl DomainEvent {
    pub fn topic(&self) -> Topic {
        self.into()
    }
}
