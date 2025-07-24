#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum StatsPeriod {
    #[default]
    Day,
    Week,
    Month,
    Year,
}

impl StatsPeriod {
    pub fn next(&self) -> Self {
        use StatsPeriod::*;
        match self {
            Day => Week,
            Week => Month,
            Month => Year,
            Year => Day,
        }
    }

    pub fn prev(&self) -> Self {
        use StatsPeriod::*;
        match self {
            Day => Year,
            Week => Day,
            Month => Week,
            Year => Month,
        }
    }
}
