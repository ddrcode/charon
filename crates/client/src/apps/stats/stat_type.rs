#[derive(Debug, Default, PartialEq)]
pub enum StatType {
    #[default]
    Wpm,
    TotalKeyPress,
}

impl StatType {
    pub fn next(&self) -> StatType {
        match self {
            StatType::Wpm => Self::TotalKeyPress,
            StatType::TotalKeyPress => Self::Wpm,
        }
    }
}
