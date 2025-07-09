use charon_lib::event::Event;
use evdev::KeyCode;
use thiserror;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, Clone, thiserror::Error)]
pub enum KOSError {
    #[error("Couldn't handle the keycode: {0:?}")]
    UnsupportedKeyCode(KeyCode),

    #[error("Couldn't produce key sequence for char: {0}")]
    UnsupportedCharacter(char),

    #[error("Unhandled device event: {0}")]
    UnhandledDeviceEvent(String),

    #[error("Event channel error: {0}")]
    EventChannelError(#[from] SendError<Event>),
}
