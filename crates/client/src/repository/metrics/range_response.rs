use std::ops::Range;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RangeResponse {
    pub status: String,
    pub data: RangeData,
}

impl RangeResponse {
    pub fn into_vec(&self) -> Vec<(f64, Option<f64>)> {
        let data = &self.data.result;
        if data.is_empty() {
            return Vec::new();
        }
        data[0]
            .values
            .iter()
            .map(|(ts, val)| (*ts, val.parse().ok()))
            .collect()
    }

    pub fn normalize_with_zeros(&self, range: Range<u64>, dataset_size: usize) -> Vec<(f64, f64)> {
        let mut slots = vec![0f64; dataset_size];
        let max = (range.end - range.start) as f64;
        let rows = self.into_vec();
        for (ts, val) in rows.into_iter() {
            let idx = (((ts - range.start as f64) * dataset_size as f64) / max)
                .floor()
                .clamp(0.0, (dataset_size - 1) as f64) as usize;
            slots[idx] = val.unwrap_or(0.0);
        }
        slots
            .iter()
            .enumerate()
            .map(|(idx, val)| (idx as f64, *val))
            .collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct RangeData {
    pub result: Vec<RangeResult>,
}

#[derive(Debug, Deserialize)]
pub struct RangeResult {
    pub metric: serde_json::Value,
    pub values: Vec<(f64, String)>,
}
