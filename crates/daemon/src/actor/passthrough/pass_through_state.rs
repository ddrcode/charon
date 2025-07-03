use crate::domain::HidKeyCode;

pub struct PassThroughState {
    modifiers: u8,
    keys: Vec<u8>,
}

impl PassThroughState {
    pub fn new() -> Self {
        PassThroughState {
            modifiers: 0,
            keys: Vec::with_capacity(6),
        }
    }

    pub fn update_on_press(&mut self, key: HidKeyCode) {
        if key.is_modifier() {
            self.modifiers |= key.modifier_mask();
        } else {
            let code = key.code();
            if !self.keys.contains(&code) && self.keys.len() < 6 {
                self.keys.push(code);
            }
        }
    }

    pub fn update_on_release(&mut self, key: HidKeyCode) {
        if key.is_modifier() {
            self.modifiers &= !key.modifier_mask();
        } else {
            let code = key.code();
            self.keys.retain(|&k| k != code);
        }
    }

    pub fn to_report(&self) -> [u8; 8] {
        let mut report = [0u8; 8];
        report[0] = self.modifiers;
        report[1] = 0;
        for (i, &k) in self.keys.iter().enumerate().take(6) {
            report[2 + i] = k;
        }
        report
    }

    pub fn reset(&mut self) {
        self.modifiers = 0;
        self.keys.clear();
    }
}
