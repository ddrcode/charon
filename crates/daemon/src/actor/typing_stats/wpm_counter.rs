use evdev::KeyCode;
use std::time::Duration;

static EXCLUDED_KEYS: &'static [u16] = &[
    14, 29, 42, 54, 56, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
];

pub struct WPMCounter {
    current_count: u16,
    current_slot: usize,
    slots: Vec<u16>,
    num_of_slots: usize,
    period: Duration,
    max_wpm: u16,
}

impl WPMCounter {
    pub fn new(period: Duration, num_of_slots: usize) -> Self {
        Self {
            current_count: 0,
            current_slot: 0,
            slots: Vec::with_capacity(num_of_slots),
            num_of_slots,
            period,
            max_wpm: 0,
        }
    }

    pub fn next(&mut self) {
        if self.slots.len() < self.num_of_slots {
            self.slots.push(self.current_count);
        } else {
            self.slots[self.current_slot] = self.current_count;
        }

        let current_wpm = self.wpm();
        if current_wpm > self.max_wpm {
            self.max_wpm = current_wpm;
        }

        self.current_count = 0;
        self.current_slot = (self.current_slot + 1) % self.num_of_slots;
    }

    pub fn register_key(&mut self, key: &KeyCode) {
        let code = key.code();
        if code > 1 && code <= 83 && !EXCLUDED_KEYS.contains(&code) {
            self.current_count += 1;
        }
    }

    pub fn wpm(&self) -> u16 {
        let sum: u16 = self.slots.iter().sum();
        sum / (5 * ((self.period.as_secs() / 60) as u16).max(1))
    }

    pub fn max_wpm(&self) -> u16 {
        self.max_wpm
    }

    pub fn reset(&mut self) {
        self.current_count = 0;
        self.current_slot = 0;
        self.slots.clear();
        self.max_wpm = 0;
    }

    pub fn period(&self) -> Duration {
        self.period
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn register_word(wpm: &mut WPMCounter) {
        wpm.register_key(&KeyCode::KEY_W);
        wpm.register_key(&KeyCode::KEY_O);
        wpm.register_key(&KeyCode::KEY_R);
        wpm.register_key(&KeyCode::KEY_D);
        wpm.register_key(&KeyCode::KEY_SPACE);
    }

    #[test]
    fn test_60s_period() {
        let mut wpm = WPMCounter::new(Duration::from_secs(60), 6);

        register_word(&mut wpm);
        wpm.next();
        assert_eq!(1, wpm.wpm());
        assert_eq!(1, wpm.max_wpm());

        register_word(&mut wpm);
        wpm.next();
        assert_eq!(2, wpm.wpm());
        assert_eq!(2, wpm.max_wpm());

        wpm.next();
        wpm.next();
        wpm.next();
        wpm.next();
        wpm.next();

        assert_eq!(1, wpm.wpm());
        assert_eq!(2, wpm.max_wpm());
    }

    #[test]
    fn test_30s_period() {
        let mut wpm = WPMCounter::new(Duration::from_secs(30), 3);

        register_word(&mut wpm);
        wpm.next();
        assert_eq!(1, wpm.wpm());
        assert_eq!(1, wpm.max_wpm());

        register_word(&mut wpm);
        wpm.next();
        assert_eq!(2, wpm.wpm());
        assert_eq!(2, wpm.max_wpm());

        wpm.next();
        wpm.next();
        wpm.next();
        wpm.next();
        wpm.next();

        assert_eq!(0, wpm.wpm());
        assert_eq!(2, wpm.max_wpm());
    }
}
