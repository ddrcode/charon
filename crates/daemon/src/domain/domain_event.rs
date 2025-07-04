use std::borrow::Cow;

use super::Mode;
use evdev::KeyCode;

#[derive(Debug, Clone)]
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
