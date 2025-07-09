use crate::domain::Topic;

use super::Mode;
use evdev::KeyCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DomainEvent {
    KeyPress(KeyCode),
    KeyRelease(KeyCode),
    HidReport([u8; 8]),
    SendText(String),
    SendFile(String),
    ModeChange(Mode),
    Exit,
}

impl DomainEvent {
    pub fn topic(&self) -> Topic {
        self.into()
    }
}
