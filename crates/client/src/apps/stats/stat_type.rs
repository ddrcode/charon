use std::fmt;

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

impl fmt::Display for StatType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            StatType::Wpm => "WPM",
            StatType::TotalKeyPress => "Key Presses",
        };
        write!(f, "{name}")
    }
}
