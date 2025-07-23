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
}

impl MetricsRepository {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:9090/api/v1".into(),
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
            "{}/query_range?query=wpm&start={start}&end={end}&step={step}s",
            self.base_url
        );
        let resp = client.get(&query).send().await?;
        let parsed = resp.json::<RangeResponse>().await?;

        Ok(parsed)
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

    fn into_vec(&self, data: &RangeResponse) -> Vec<(f64, f64)> {
        let data = &data.data.result;
        if data.is_empty() {
            return Vec::new();
        }
        data[0]
            .values
            .iter()
            .enumerate()
            .map(|(i, (_, val))| (i as f64, val.parse().unwrap()))
            .collect()
    }
}
