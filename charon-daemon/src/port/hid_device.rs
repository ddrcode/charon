// SPDX-License-Identifier: GPL-3.0-or-later
pub trait HIDDevice: Send + 'static {
    fn send_report(&mut self, report: &[u8; 8]) -> std::io::Result<()>;

    fn reset(&mut self) -> std::io::Result<()> {
        self.send_report(&[0u8; 8])
    }
}
