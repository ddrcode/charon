use std::borrow::Cow;

use crate::error::KOSError;

use super::Mode;
use evdev::KeyCode;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum DomainEvent {
    KeyPress(KeyCode),
    KeyRelease(KeyCode),
    ModeChange(Mode),
    Warning(Cow<'static, str>),
    Error(KOSError),
}
