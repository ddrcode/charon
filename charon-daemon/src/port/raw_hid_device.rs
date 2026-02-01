// SPDX-License-Identifier: GPL-3.0-or-later
use crate::error::CharonError;

pub trait RawHidDevice {
    fn read_packet(&mut self) -> impl Future<Output = Result<(usize, [u8; 32]), CharonError>>;
}
