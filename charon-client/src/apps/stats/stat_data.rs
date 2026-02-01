// SPDX-License-Identifier: GPL-3.0-or-later
use std::collections::HashMap;

#[derive(Debug)]
pub enum StatData {
    TimeSeries(Vec<Vec<(f64, f64)>>),
    Frequency(HashMap<String, f64>),
}

impl Default for StatData {
    fn default() -> Self {
        StatData::TimeSeries(Vec::new())
    }
}

impl From<Vec<Option<Vec<(f64, f64)>>>> for StatData {
    fn from(data: Vec<Option<Vec<(f64, f64)>>>) -> Self {
        StatData::TimeSeries(data.into_iter().flatten().collect())
    }
}
