use super::{HidKeyCode, Modifiers};

pub struct KeyboardState {
    modifiers: Modifiers,
    keys: Vec<HidKeyCode>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            modifiers: Modifiers::default(),
            keys: Vec::with_capacity(6),
        }
    }

    pub fn update_on_press(&mut self, key: HidKeyCode) {
        if key.is_modifier() {
            self.modifiers.add(key.into());
        } else {
            if !self.keys.contains(&key) && self.keys.len() < 6 {
                self.keys.push(key);
            }
        }
    }

    pub fn update_on_release(&mut self, key: HidKeyCode) {
        if key.is_modifier() {
            self.modifiers.remove(key.into());
        } else {
            self.keys.retain(|&k| k != key);
        }
    }

    pub fn to_report(&self) -> [u8; 8] {
        let mut report = [0u8; 8];
        report[0] = self.modifiers.value();
        report[1] = 0;
        for (i, &k) in self.keys.iter().enumerate().take(6) {
            report[2 + i] = k.code();
        }
        report
    }

    pub fn reset(&mut self) {
        self.modifiers.reset();
        self.keys.clear();
    }

    pub fn is(&self, key: HidKeyCode, modifiers: Modifiers) -> bool {
        self.modifiers == modifiers && self.keys.len() == 1 && self.keys[0] == key.into()
    }
}
