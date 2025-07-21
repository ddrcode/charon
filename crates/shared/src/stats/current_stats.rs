use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentStats {
    pub total: u64,
    pub wpm: u16,
}

impl CurrentStats {
    pub fn new(total: u64, wpm: u16) -> Self {
        Self { total, wpm }
    }
}
