use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentStats {
    pub today: u64,
    pub total: u64,
    pub wpm: u16,
    pub max_wpm: u16,
}

impl CurrentStats {
    pub fn new(today: u64, total: u64, wpm: u16, max_wpm: u16) -> Self {
        Self {
            today,
            total,
            wpm,
            max_wpm,
        }
    }
}
