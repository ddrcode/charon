use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    PassThrough,
    InApp,
}

impl Mode {
    pub fn toggle(&self) -> Mode {
        match self {
            Mode::InApp => Mode::PassThrough,
            Mode::PassThrough => Mode::InApp,
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::PassThrough
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mode = match self {
            Mode::InApp => "in-app",
            Mode::PassThrough => "pass-through",
        };
        write!(f, "{mode}")
    }
}
