use chrono::{Days, Months, offset::LocalResult, prelude::*};
use tracing::warn;

use crate::apps::stats::StatsPeriod;

#[derive(Debug, Default)]
pub struct State {
    pub start: u64,
    pub period: StatsPeriod,
    pub resolution: usize,
    pub shift: u16,
    pub data1: Option<Vec<(f64, f64)>>,
    pub data2: Option<Vec<(f64, f64)>>,
}

impl State {
    pub fn end(&self) -> u64 {
        self.start + self.sec_per_period() - 1
    }

    pub fn step(&self) -> u64 {
        self.sec_per_period() / self.resolution as u64
    }

    pub fn start_end_step(&self) -> (u64, u64, u64) {
        (self.start, self.end(), self.step())
    }

    pub fn prev(&mut self) {
        let date: DateTime<Local> = Local.timestamp_opt(self.start as i64, 0).unwrap();
        let maybe_new_start = match self.period {
            StatsPeriod::Day => date.checked_sub_days(Days::new(1)),
            StatsPeriod::Week => date.checked_sub_days(Days::new(7)),
            StatsPeriod::Month => date.checked_sub_months(Months::new(1)),
            StatsPeriod::Year => date.checked_sub_months(Months::new(12)),
        };
        if let Some(new_start) = maybe_new_start
            && new_start.year() > 2020
        {
            self.shift += 1;
            self.start = new_start.timestamp() as u64;
        } else {
            warn!("Couldn't compute new start for {:?}", maybe_new_start);
        }
    }

    pub fn next(&mut self) {
        if self.shift == 0 {
            return;
        }
        let date: DateTime<Local> = Local.timestamp_opt(self.start as i64, 0).unwrap();
        let maybe_new_start = match self.period {
            StatsPeriod::Day => date.checked_add_days(Days::new(1)),
            StatsPeriod::Week => date.checked_add_days(Days::new(7)),
            StatsPeriod::Month => date.checked_add_months(Months::new(1)),
            StatsPeriod::Year => date.checked_add_months(Months::new(12)),
        };
        if let Some(new_start) = maybe_new_start
            && new_start.year() > 2020
        {
            self.shift -= 1;
            self.start = new_start.timestamp() as u64;
        } else {
            warn!("Couldn't compute new start for {:?}", maybe_new_start);
        }
    }

    pub fn reset_with_period(&mut self, period: StatsPeriod) {
        self.period = period;
        let now = Local::now();
        let date = match period {
            StatsPeriod::Day => Local.with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0),
            StatsPeriod::Week => {
                // todo!()
                let today_midnight = Local
                    .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
                    .unwrap();
                let days_since_monday = today_midnight.weekday().num_days_from_monday() as i64;
                LocalResult::Single(today_midnight - chrono::Duration::days(days_since_monday))
            }
            StatsPeriod::Month => Local.with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0),
            StatsPeriod::Year => Local.with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0),
        }
        .unwrap();
        self.start = date.timestamp() as u64;
    }

    pub fn sec_per_period(&self) -> u64 {
        let date: DateTime<Local> = Local.timestamp_opt(self.start as i64, 0).unwrap();
        let days: u64 = match self.period {
            StatsPeriod::Day => 1,
            StatsPeriod::Week => 7,
            StatsPeriod::Month => date.num_days_in_month().into(),
            StatsPeriod::Year => {
                if date.date_naive().leap_year() {
                    366
                } else {
                    365
                }
            }
        };
        days * 3600 * 24
    }
}
