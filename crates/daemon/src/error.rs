use charon_lib::event::Event;
use evdev::KeyCode;
use thiserror;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, thiserror::Error)]
pub enum KOSError {
    #[error("Couldn't handle the keycode: {0:?}")]
    UnsupportedKeyCode(KeyCode),

    #[error("Couldn't produce key sequence for char: {0}")]
    UnsupportedCharacter(char),

    #[error("Unhandled device event: {0}")]
    UnhandledDeviceEvent(String),

    #[error("Invalid key shortcut: {0}")]
    InvalidKeyShortcut(String),

    #[error("Unsupported key name: {0}")]
    UnsupportedKeyName(String),

    #[error("Event channel error: {0}")]
    EventChannelError(#[from] SendError<Event>),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}
