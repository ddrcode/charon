// SPDX-License-Identifier: GPL-3.0-or-later
use evdev::KeyCode;
use serde::{Deserialize, Serialize};

use super::{Mode, Topic};
use super::{qmk::QMKEvent, stats::CurrentStats};

#[derive(maiko::Event, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CharonEvent {
    KeyPress(KeyCode, String),
    KeyRelease(KeyCode, String),
    HidReport([u8; 8]),
    SendText(String),
    SendFile(String, bool),
    TextSent,
    CurrentStats(CurrentStats),

    // system events
    ModeChange(Mode),
    Sleep,
    WakeUp,

    // telemetry events
    ReportSent,

    // QMK
    QMKEvent(QMKEvent),
}

impl CharonEvent {
    pub fn topic(&self) -> Topic {
        self.into()
    }
}
