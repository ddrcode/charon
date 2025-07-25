use reqwest::Client;
use tracing::{debug, info};

use super::{QueryBuilder, RangeResponse};

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
        // "{}/query_range?query=avg_over_time(wpm[{step}s])&start={start}&end={end}&step={step}s",
        QueryBuilder::new(&self.client, &self.base_url)
            .query("wpm")
            .range(start, end, step)
            .send::<RangeResponse>()
            .await
    }

    pub async fn max_wpm_for_range(
        &self,
        start: u64,
        end: u64,
        step: u64,
    ) -> anyhow::Result<RangeResponse> {
        QueryBuilder::new(&self.client, &self.base_url)
            .query(format!("max_over_time(wpm[{step}s])"))
            .range(start, end, step)
            .send::<RangeResponse>()
            .await
    }

    pub async fn total_key_presses_for_range(
        &self,
        start: u64,
        end: u64,
        step: u64,
    ) -> anyhow::Result<RangeResponse> {
        QueryBuilder::new(&self.client, &self.base_url)
            .query(format!(
                "sum with(user) increase(key_presses_total[{step}s])"
            ))
            .range(start, end, step)
            .send::<RangeResponse>()
            .await
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
        let rows = data.into_vec();
        for (ts, val) in rows.into_iter() {
            let idx = (((ts - start as f64) * self.dataset_size as f64) / max)
                .floor()
                .clamp(0.0, (self.dataset_size - 1) as f64) as usize;
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
