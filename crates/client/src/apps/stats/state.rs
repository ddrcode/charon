use crate::apps::stats::StatsPeriod;

#[derive(Debug, Default)]
pub struct State {
    pub start: u64,
    pub period: StatsPeriod,
    pub resolution: u64,
    pub shift: u16,
    pub data1: Option<Vec<(f64, f64)>>,
    pub data2: Option<Vec<(f64, f64)>>,
}

impl State {
    pub fn end(&self) -> u64 {
        self.start + self.period.val()
    }

    pub fn step(&self) -> u64 {
        let val = self.period.val();
        val / self.resolution
    }

    pub fn prev(&mut self) {
        self.shift += 1;
        self.start -= self.period.val();
    }

    pub fn next(&mut self) {
        if self.shift > 0 {
            self.shift -= 1;
            self.start += self.period.val();
        }
    }
}
