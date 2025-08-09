use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct QMKRecord {
    pub keycode: u16,
    pub pressed: bool,
    pub row: u8,
    pub col: u8,
}

impl QMKRecord {
    pub fn new(keycode: u16, pressed: bool, row: u8, col: u8) -> Self {
        Self {
            keycode,
            pressed,
            row,
            col,
        }
    }
}
