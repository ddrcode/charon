// SPDX-License-Identifier: GPL-3.0-or-later
use serde::{Deserialize, Serialize};
use strum::{Display, FromRepr};

#[repr(u8)]
#[derive(Debug, Default, Display, FromRepr, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    #[strum(to_string = "pass-through")]
    PassThrough = 0,

    #[strum(to_string = "in-app")]
    InApp = 1,
}

impl Mode {
    pub fn toggle(&self) -> Mode {
        match self {
            Mode::InApp => Mode::PassThrough,
            Mode::PassThrough => Mode::InApp,
        }
    }
}
