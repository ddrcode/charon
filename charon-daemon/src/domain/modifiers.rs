// SPDX-License-Identifier: GPL-3.0-or-later
use crate::domain::HidKeyCode;
use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(transparent)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const NONE: Self = Self(0);
    pub const LEFT_CTRL: Self = Self(1);
    pub const LEFT_SHIFT: Self = Self(2);
    pub const LEFT_ALT: Self = Self(4);
    pub const LEFT_META: Self = Self(8);
    pub const RIGHT_CTRL: Self = Self(16);
    pub const RIGHT_SHIFT: Self = Self(32);
    pub const RIGHT_ALT: Self = Self(64);
    pub const RIGHT_META: Self = Self(128);

    pub fn new(val: u8) -> Self {
        Self(val)
    }

    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn add(&mut self, modifiers: Modifiers) {
        self.0 |= modifiers.value();
    }

    pub fn remove(&mut self, modifiers: Modifiers) {
        self.0 &= !modifiers.value();
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }
}

impl From<Modifiers> for u8 {
    fn from(value: Modifiers) -> Self {
        value.value()
    }
}

impl From<u8> for Modifiers {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<HidKeyCode> for Modifiers {
    fn from(code: HidKeyCode) -> Self {
        Self(code.modifier_mask())
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut mods: Vec<String> = Vec::new();
        for i in 0..4 {
            if self.0 & !((1 + 16) << i) != 0 {
                mods.push(
                    (match i {
                        0 => "Ctrl",
                        1 => "Shift",
                        2 => "Alt",
                        3 => "Meta",
                        _ => unreachable!("There are only 4 types of modifiers"),
                    })
                    .to_string(),
                );
            }
        }
        write!(f, "{}", mods.join("+"))
    }
}
