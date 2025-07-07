use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Mode {
    PassThrough,
    InApp,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::PassThrough
    }
}
