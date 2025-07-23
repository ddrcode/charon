use crate::{apps::stats::StatsPeriod, repository::metrics::RangeResponse};

#[derive(Debug, Default)]
pub struct State {
    pub start: u64,
    pub period: StatsPeriod,
    pub resolution: u64,
    pub data1: Option<RangeResponse>,
    pub data2: Option<RangeResponse>,
}

impl State {
    pub fn end(&self) -> u64 {
        self.start + self.period.val()
    }

    pub fn step(&self) -> u64 {
        let val = self.period.val();
        val / self.resolution
    }
}
