use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::{domain::HidReport, port::HIDDevice};

pub struct HidDeviceMock {
    state: Arc<Mutex<VecDeque<HidReport>>>,
}

impl HidDeviceMock {
    pub fn new(state: Arc<Mutex<VecDeque<HidReport>>>) -> Self {
        Self { state }
    }
}

impl HIDDevice for HidDeviceMock {
    fn send_report(&mut self, report: &[u8; 8]) -> std::io::Result<()> {
        match self.state.lock().as_mut() {
            Ok(state) => state.push_back(HidReport::from(report)),
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ResourceBusy,
                    e.to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl Default for HidDeviceMock {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}
