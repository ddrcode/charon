use std::{
    fs::{File, OpenOptions},
    io::Write,
};

use tracing::error;

pub struct HIDKeyboard {
    hidg: File,
}

impl HIDKeyboard {
    pub fn new(path: &str) -> Self {
        let hidg = OpenOptions::new()
            .write(true)
            .open(path)
            .expect("Failed to open HID gadget device");

        Self { hidg }
    }

    pub fn send_report(&mut self, report: &[u8; 8]) {
        if let Err(e) = self.hidg.write_all(report) {
            error!("Failed to write HID report: {}", e);
        }
    }

    pub fn reset(&mut self) {
        self.send_report(&[0u8; 8]);
    }
}
