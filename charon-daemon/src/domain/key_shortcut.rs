// SPDX-License-Identifier: GPL-3.0-or-later
use std::{fmt, str::FromStr};

use crate::{domain::HidReport, error::CharonError};

use super::{HidKeyCode, Modifiers};

#[derive(Debug, Clone, PartialEq)]
pub struct KeyShortcut {
    modifiers: Modifiers,
    key: HidKeyCode,
}

impl KeyShortcut {
    pub fn new(key: HidKeyCode, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }
}

impl FromStr for KeyShortcut {
    type Err = CharonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return Err(CharonError::InvalidKeyShortcut(s.into()));
        }

        let mut modifiers = Modifiers::default();
        let key_part = parts.last().unwrap();

        if parts.len() > 1 {
            for part in &parts[0..parts.len() - 1] {
                match part.to_lowercase().as_str() {
                    "ctrl" => modifiers.add(Modifiers::LEFT_CTRL),
                    "shift" => modifiers.add(Modifiers::LEFT_SHIFT),
                    "alt" => modifiers.add(Modifiers::LEFT_ALT),
                    "meta" | "cmd" | "super" => modifiers.add(Modifiers::LEFT_META),
                    _ => return Err(CharonError::InvalidKeyShortcut(s.into())),
                }
            }
        }

        let key = HidKeyCode::from_str(key_part)?;

        Ok(KeyShortcut::new(key, modifiers))
    }
}

impl fmt::Display for KeyShortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}+{}", self.modifiers, self.key)
    }
}

impl From<&KeyShortcut> for u64 {
    fn from(key: &KeyShortcut) -> Self {
        let mut bytes = [0u8; 8];
        bytes[0] = key.modifiers.into();
        bytes[2] = key.key.into();
        u64::from_ne_bytes(bytes)
    }
}

impl From<KeyShortcut> for u64 {
    fn from(key: KeyShortcut) -> Self {
        u64::from(&key)
    }
}

impl From<&KeyShortcut> for HidReport {
    fn from(key: &KeyShortcut) -> Self {
        let mut bytes = [0u8; 8];
        bytes[0] = key.modifiers.into();
        bytes[2] = key.key.into();
        HidReport::new(bytes)
    }
}
