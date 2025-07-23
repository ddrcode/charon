#[derive(Debug, Default, PartialEq)]
pub enum StatsPeriod {
    #[default]
    Day,
    Week,
    Month,
    Year,
}

impl StatsPeriod {
    pub fn val(&self) -> u64 {
        u64::from(self)
    }

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

impl From<&StatsPeriod> for u64 {
    fn from(value: &StatsPeriod) -> Self {
        match value {
            StatsPeriod::Day => 3600 * 24,
            StatsPeriod::Week => 3600 * 24 * 7,
            StatsPeriod::Month => 3600 * 24 * 30,
            StatsPeriod::Year => 3600 * 24 * 365,
        }
    }
}
