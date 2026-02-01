// SPDX-License-Identifier: GPL-3.0-or-later
#[derive(Debug, Default, Clone, Copy)]
#[repr(transparent)]
pub struct HidReport([u8; 8]);

impl HidReport {
    pub fn new(report: [u8; 8]) -> Self {
        Self(report)
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        self.0
    }
}

impl From<&HidReport> for [u8; 8] {
    fn from(report: &HidReport) -> Self {
        report.to_bytes()
    }
}

impl From<HidReport> for [u8; 8] {
    fn from(report: HidReport) -> Self {
        report.to_bytes()
    }
}
