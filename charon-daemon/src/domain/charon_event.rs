// SPDX-License-Identifier: GPL-3.0-or-later
use evdev::KeyCode;
use maiko::{Event, Label};
use serde::{Deserialize, Serialize};

use super::{Mode, Topic};
use super::{qmk::QMKEvent, stats::CurrentStats};

#[derive(Event, Label, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CharonEvent {
    // Typing
    KeyPress(KeyCode, String),
    KeyRelease(KeyCode, String),
    HidReport([u8; 8]),
    SendText(String),
    SendFile(String, bool),
    TextSent,

    // Keyboard
    KeyboardAttached(String),

    // Stats and telemetry
    CurrentStats(CurrentStats),
    ReportSent,

    // System events
    ModeChange(Mode),
    Sleep,
    WakeUp,

    // QMK
    QMKEvent(QMKEvent),
}

impl CharonEvent {
    pub fn topic(&self) -> Topic {
        self.into()
    }
}
