use evdev::KeyCode;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum CharonError {
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

    #[error("Couldn't find requested keyboard: {0}")]
    KeyboardNotFound(String),

    #[error("QMK error: {0}")]
    QMKError(String),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Prometheus error: {0}")]
    PrometheusError(#[from] prometheus::Error),

    #[error("Yaml parsing error: {0}")]
    YamlError(#[from] serde_yaml_bw::Error),

    #[error("Actor handling error: {0}")]
    MaikoError(#[from] maiko::Error),

    #[error("RawHid error: {0}")]
    RawHidError(#[from] async_hid::HidError),
}

impl From<CharonError> for maiko::Error {
    fn from(err: CharonError) -> Self {
        maiko::Error::External(err.to_string().into())
    }
}
