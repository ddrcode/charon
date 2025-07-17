use std::path::PathBuf;

use crate::domain::{HidKeyCode, KeyShortcut, Modifiers};

pub(crate) fn default_hid_keyboard() -> PathBuf {
    PathBuf::from("/dev/hidg0")
}

pub(crate) fn default_typing_interval() -> u8 {
    20
}

pub(crate) fn default_server_socket() -> PathBuf {
    PathBuf::from("/tmp/charon.sock")
}

pub(crate) fn default_channel_size() -> usize {
    128
}

pub fn default_quit_shortcut() -> KeyShortcut {
    KeyShortcut::new(HidKeyCode::KEY_Q, Modifiers::LEFT_CTRL)
}

pub fn default_toggle_mode_shortcut() -> KeyShortcut {
    KeyShortcut::new(HidKeyCode::KEY_F7, Modifiers::NONE)
}

pub fn default_awake_host_shortcut() -> KeyShortcut {
    KeyShortcut::new(HidKeyCode::KEY_F8, Modifiers::NONE)
}

pub fn default_time_to_sleep() -> u64 {
    900
}
