use evdev::KeyCode;
use serde::{Deserialize, Serialize};

use super::{Mode, Topic};
use crate::{qmk::QMKEvent, stats::CurrentStats};

#[derive(maiko::Event, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DomainEvent {
    KeyPress(KeyCode, String),
    KeyRelease(KeyCode, String),
    HidReport([u8; 8]),
    SendText(String),
    SendFile(String, bool),
    TextSent,
    CurrentStats(CurrentStats),

    // system events
    ModeChange(Mode),
    Exit,
    Sleep,
    WakeUp,

    // telemetry events
    ReportSent,

    // QMK
    QMKEvent(QMKEvent),
}

impl DomainEvent {
    pub fn topic(&self) -> Topic {
        self.into()
    }
}
