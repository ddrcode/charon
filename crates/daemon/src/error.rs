use evdev::KeyCode;
use thiserror;

#[derive(Debug, Clone, thiserror::Error)]
pub enum KOSError {
    #[error("Couldn't handle the keycode: {0:?}")]
    UnsupportedKeyCode(KeyCode),

    #[error("Unhandled device event: {0}")]
    UnhandledDeviceEvent(String),
}
