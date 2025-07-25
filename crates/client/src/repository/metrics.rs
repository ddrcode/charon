use reqwest::Client;
use serde::Deserialize;
use tracing::debug;

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

pub struct MetricsRepository {
    pub base_url: String,
    pub dataset_size: usize,
    pub client: Client,
}

impl MetricsRepository {
    pub fn new(dataset_size: usize) -> Self {
        Self {
            base_url: "http://localhost:9090/api/v1".into(),
            dataset_size,
            client: Client::new(),
        }
    }

    pub async fn avg_wpm_for_range(
        &self,
        start: u64,
        end: u64,
        step: u64,
    ) -> anyhow::Result<RangeResponse> {
        let query = format!(
            // "{}/query_range?query=avg_over_time(wpm[{step}s])&start={start}&end={end}&step={step}s",
            "{}/query_range?query=wpm&start={start}&end={end}&step={step}s",
            self.base_url
        );
        debug!("Sending query: {query}");
        let resp = self.client.get(&query).send().await?;
        let parsed = resp.json::<RangeResponse>().await?;

        Ok(parsed)
    }

    pub async fn max_wpm_for_range(
        &self,
        start: u64,
        end: u64,
        step: u64,
    ) -> anyhow::Result<RangeResponse> {
        let query = format!(
            "{}/query_range?query=max_over_time(wpx[{step}s])&start={start}&end={end}&step={step}s",
            self.base_url
        );
        debug!("Sending query: {query}");
        let resp = self.client.get(&query).send().await?;
        let parsed = resp.json::<RangeResponse>().await?;

        Ok(parsed)
    }

    pub fn normalize_with_zeros(
        &self,
        data: anyhow::Result<RangeResponse>,
        start: u64,
        end: u64,
    ) -> anyhow::Result<Vec<(f64, f64)>> {
        let data = data?;
        let mut slots = vec![0f64; self.dataset_size];
        let max = (end - start) as f64;
        for (ts, val) in data.into_vec().into_iter() {
            let idx = (((ts - start as f64) * self.dataset_size as f64) / max)
                .floor()
                .clamp(0.0, self.dataset_size as f64) as usize;
            slots[idx] = val.unwrap_or(0.0);
        }
        let result = slots
            .iter()
            .enumerate()
            .map(|(idx, val)| (idx as f64, *val))
            .collect();
        Ok(result)
    }
}
