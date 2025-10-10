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

    pub fn to_bytes(&self) -> [u8; 5] {
        let kc = self.keycode.to_le_bytes();
        [kc[0], kc[1], self.pressed.into(), self.row, self.col]
    }
}
