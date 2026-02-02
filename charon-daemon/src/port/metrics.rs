// SPDX-License-Identifier: GPL-3.0-or-later
use core::future::Future;

use evdev::KeyCode;

use crate::error::CharonError;

pub trait Metrics: Send + 'static {
    fn register_key_event(&self, key: &KeyCode, keyboard: &str);
    fn register_key_to_report_time(&self, time: u64);
    fn register_wpm(&self, wpm: u16);

    fn flush(&mut self) -> impl Future<Output = Result<(), CharonError>> + Send;
}
