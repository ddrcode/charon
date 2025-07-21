use evdev::KeyCode;

static EXCLUDED_KEYS: &'static [u16] = &[
    14, 29, 42, 54, 56, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
];

pub struct WPMCounter {
    current_count: u16,
    current_slot: usize,
    slots: [u16; 12],
}

impl WPMCounter {
    pub fn new() -> Self {
        Self {
            current_count: 0,
            current_slot: 0,
            slots: [0; 12],
        }
    }

    pub fn next(&mut self) {
        self.slots[self.current_slot] = self.current_count;
        self.current_count = 0;
        self.current_slot = (self.current_slot + 1) % 12;
    }

    pub fn register_key(&mut self, key: &KeyCode) {
        let code = key.code();
        if code > 1 && code <= 83 && !EXCLUDED_KEYS.contains(&code) {
            self.current_count += 1;
        }
    }

    pub fn wpm(&self) -> u16 {
        let sum: u16 = self.slots.iter().sum();
        sum / 5
    }
}
