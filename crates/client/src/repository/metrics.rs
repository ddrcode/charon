use reqwest::Client;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct RangeResponse {
    status: String,
    data: RangeData,
}

#[derive(Debug, Deserialize)]
pub struct RangeData {
    result: Vec<RangeResult>,
}

#[derive(Debug, Deserialize)]
pub struct RangeResult {
    metric: serde_json::Value,
    values: Vec<(f64, String)>, // (timestamp, value)
}

pub struct MetricsRepository {
    base_url: String,
    dataset_size: usize,
}

impl MetricsRepository {
    pub fn new(dataset_size: usize) -> Self {
        Self {
            base_url: "http://localhost:9090/api/v1".into(),
            dataset_size,
        }
    }

    pub async fn avg_wpm_for_range(
        &self,
        start: u64,
        end: u64,
        step: u64,
    ) -> anyhow::Result<RangeResponse> {
        let client = Client::new();
        let query = format!(
            "{}/query_range?query=avg_over_time(wpm[{step}s])&start={start}&end={end}&step={step}s",
            // "{}/query_range?query=wpm&start={start}&end={end}&step={step}s",
            self.base_url
        );
        let resp = client.get(&query).send().await?;
        let parsed = resp.json::<RangeResponse>().await?;

        Ok(parsed)
    }

    pub async fn avg_wpm_for_range_normalized(
        &self,
        start: u64,
        end: u64,
        step: u64,
    ) -> anyhow::Result<Vec<(f64, f64)>> {
        Ok(self.normalize(&self.avg_wpm_for_range(start, end, step).await?, start, end))
    }

    pub async fn max_wpm_for_range(
        &self,
        start: u64,
        end: u64,
        step: u64,
    ) -> anyhow::Result<RangeResponse> {
        let client = Client::new();
        let query = format!(
            "{}/query_range?query=max_over_time(wpm[{step}s])&start={start}&end={end}&step={step}s",
            self.base_url
        );
        let resp = client.get(&query).send().await?;
        let parsed = resp.json::<RangeResponse>().await?;

        Ok(parsed)
    }

    pub async fn max_wpm_for_range_normalized(
        &self,
        start: u64,
        end: u64,
        step: u64,
    ) -> anyhow::Result<Vec<(f64, f64)>> {
        Ok(self.normalize(&self.max_wpm_for_range(start, end, step).await?, start, end))
    }

    fn into_vec(&self, data: &RangeResponse) -> Vec<(f64, f64)> {
        let data = &data.data.result;
        if data.is_empty() {
            return Vec::new();
        }
        data[0]
            .values
            .iter()
            .map(|(ts, val)| (*ts, val.parse().unwrap()))
            .collect()
    }

    fn normalize(&self, data: &RangeResponse, start: u64, end: u64) -> Vec<(f64, f64)> {
        let mut slots = vec![0f64; self.dataset_size + 1];
        let max = (end - start) as f64;
        for (ts, val) in self.into_vec(data).into_iter() {
            let idx = ((ts - start as f64) * (self.dataset_size as f64)) / max;
            slots[idx as usize] = val;
        }
        slots
            .iter()
            .enumerate()
            .map(|(idx, val)| (idx as f64, *val))
            .collect()
    }
}
