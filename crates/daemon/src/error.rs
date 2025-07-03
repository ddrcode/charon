use evdev::KeyCode;
use thiserror;

use crate::domain::Event;

#[derive(Debug, Clone, thiserror::Error)]
pub enum KOSError {
    #[error("Couldn't handle the keycode: {0:?}")]
    UnsupportedKeyCode(KeyCode),

    #[error("Unhandled device event: {0}")]
    UnhandledDeviceEvent(String),

    #[error("Event channel error: {0}")]
    EventChannelError(#[from] crossbeam_channel::SendError<Event>),
}
