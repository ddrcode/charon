// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use crate::port::HIDDevice;

pub struct HIDDeviceUnix {
    hidg: File,
}

impl HIDDeviceUnix {
    pub fn new(path: &Path) -> Self {
        let hidg = OpenOptions::new()
            .write(true)
            .open(path)
            .expect("Failed to open HID gadget device");

        Self { hidg }
    }
}

impl HIDDevice for HIDDeviceUnix {
    fn send_report(&mut self, report: &[u8; 8]) -> std::io::Result<()> {
        self.hidg.write_all(report)
    }
}

impl Drop for HIDDeviceUnix {
    fn drop(&mut self) {
        let _ = self.reset();
    }
}
