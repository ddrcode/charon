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
