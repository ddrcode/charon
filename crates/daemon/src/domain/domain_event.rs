use std::borrow::Cow;

use super::Mode;
use evdev::KeyCode;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub enum DomainEvent {
    KeyPress(KeyCode),
    KeyRelease(KeyCode),
    SendText(String),
    SendFile(String),
    ModeChange(Mode),
    Warning(Cow<'static, str>),
    Exit,
}
