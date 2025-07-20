use serde::{Deserialize, Serialize};

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
