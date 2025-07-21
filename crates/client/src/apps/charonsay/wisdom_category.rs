#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum WisdomCategory {
    #[default]
    Splash,
    Idle,
    Speed,
    Charonsay,
}

impl From<WisdomCategory> for &'static str {
    fn from(value: WisdomCategory) -> Self {
        match value {
            WisdomCategory::Splash => "splash",
            WisdomCategory::Idle => "idle",
            WisdomCategory::Speed => "speed",
            WisdomCategory::Charonsay => "charonsay",
        }
    }
}
