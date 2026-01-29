use evdev::KeyCode;
use std::time::Duration;

/// Returns true if this key should count toward WPM.
/// Counts keys that produce visible characters: letters, numbers, punctuation, space.
/// Excludes modifiers, function keys, navigation, backspace, etc.
fn is_typing_key(code: u16) -> bool {
    matches!(
        code,
        2..=13      // 1234567890-=
        | 16..=27   // qwertyuiop[]
        | 28        // Enter
        | 30..=41   // asdfghjkl;'`
        | 43..=53   // \zxcvbnm,./
        | 57        // Space
    )
}

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
        if is_typing_key(key.code()) {
            self.current_count += 1;
        }
    }

    pub fn wpm(&self) -> u16 {
        let sum: u16 = self.slots.iter().sum();
        let seconds = self.slots.len() as f32 * self.period.as_secs_f32();

        if seconds == 0.0 {
            return 0;
        }

        ((sum as f32 * 60.0) / (5.0 * seconds)) as u16
    }

    pub fn max_wpm(&self) -> u16 {
        self.max_wpm
    }

    pub fn period(&self) -> Duration {
        self.period
    }

    pub fn set_wpm_max(&mut self, max: u16) {
        self.max_wpm = max;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn register_keys(wpm: &mut WPMCounter, count: usize) {
        for _ in 0..count {
            wpm.register_key(&KeyCode::KEY_A);
        }
    }

    #[test]
    fn test_wpm_basic_calculation() {
        // 5 keys in 60 seconds = 1 WPM (standard: 5 chars = 1 word)
        let mut counter = WPMCounter::new(Duration::from_secs(60), 1);
        register_keys(&mut counter, 5);
        counter.next();
        assert_eq!(1, counter.wpm());

        // 5 keys in 30 seconds = 2 WPM (double the rate)
        let mut counter = WPMCounter::new(Duration::from_secs(30), 1);
        register_keys(&mut counter, 5);
        counter.next();
        assert_eq!(2, counter.wpm());

        // 10 keys in 60 seconds = 2 WPM
        let mut counter = WPMCounter::new(Duration::from_secs(60), 1);
        register_keys(&mut counter, 10);
        counter.next();
        assert_eq!(2, counter.wpm());
    }

    #[test]
    fn test_wpm_rolling_window() {
        // 3 slots of 30s each = 90s window
        let mut counter = WPMCounter::new(Duration::from_secs(30), 3);

        // Slot 1: 5 keys in 30s window = 2 WPM
        register_keys(&mut counter, 5);
        counter.next();
        assert_eq!(2, counter.wpm());

        // Slot 2: 10 keys in 60s window = 2 WPM
        register_keys(&mut counter, 5);
        counter.next();
        assert_eq!(2, counter.wpm());

        // Slot 3: 15 keys in 90s window = 2 WPM
        register_keys(&mut counter, 5);
        counter.next();
        assert_eq!(2, counter.wpm());

        // Slot 4: empty, replaces slot 1. Now [0,5,5] = 10 keys in 90s = 1.33 -> 1 WPM
        counter.next();
        assert_eq!(1, counter.wpm());

        // Slot 5: empty, replaces slot 2. Now [0,0,5] = 5 keys in 90s = 0.67 -> 0 WPM
        counter.next();
        assert_eq!(0, counter.wpm());

        // Slot 6: empty, all zeros
        counter.next();
        assert_eq!(0, counter.wpm());
    }

    #[test]
    fn test_max_wpm_tracks_peak() {
        let mut counter = WPMCounter::new(Duration::from_secs(60), 3);

        // Slow typing: 5 keys = 1 WPM
        register_keys(&mut counter, 5);
        counter.next();
        assert_eq!(1, counter.max_wpm());

        // Burst: 15 keys in this slot, window is [5,15] = 20 keys in 120s = 2 WPM
        register_keys(&mut counter, 15);
        counter.next();
        assert_eq!(2, counter.max_wpm());

        // Slow down: current WPM drops but max stays
        register_keys(&mut counter, 5);
        counter.next();
        // Window is [5,15,5] = 25 keys in 180s = 1.67 -> 1 WPM
        assert_eq!(1, counter.wpm());
        assert_eq!(2, counter.max_wpm()); // peak preserved
    }

    #[test]
    fn test_only_typing_keys_counted() {
        let mut counter = WPMCounter::new(Duration::from_secs(60), 1);

        // These should NOT count (modifiers, function keys, navigation, backspace)
        counter.register_key(&KeyCode::KEY_BACKSPACE);
        counter.register_key(&KeyCode::KEY_LEFTCTRL);
        counter.register_key(&KeyCode::KEY_LEFTSHIFT);
        counter.register_key(&KeyCode::KEY_LEFTALT);
        counter.register_key(&KeyCode::KEY_F1);
        counter.register_key(&KeyCode::KEY_ESC);
        counter.register_key(&KeyCode::KEY_TAB);
        counter.register_key(&KeyCode::KEY_UP);

        // These SHOULD count (letters, numbers, punctuation, space, enter)
        counter.register_key(&KeyCode::KEY_A);
        counter.register_key(&KeyCode::KEY_Z);
        counter.register_key(&KeyCode::KEY_1);
        counter.register_key(&KeyCode::KEY_SPACE);
        counter.register_key(&KeyCode::KEY_ENTER);
        counter.register_key(&KeyCode::KEY_COMMA);
        counter.register_key(&KeyCode::KEY_DOT);

        counter.next();
        // 7 typing keys counted
        // 7 keys in 60s = 1.4 WPM -> 1
        assert_eq!(1, counter.wpm());
    }
}
