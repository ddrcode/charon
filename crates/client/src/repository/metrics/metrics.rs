use reqwest::Client;

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
                "sum by(user) (increase(key_presses_total[{step}s]))"
            ))
            .range(start, end, step)
            .send::<RangeResponse>()
            .await
    }

    pub async fn key_frequency_for_range(
        &self,
        start: u64,
        end: u64,
        step: u64,
    ) -> anyhow::Result<RangeResponse> {
        QueryBuilder::new(&self.client, &self.base_url)
            .query(format!(
                "sum by(user, key) (increase(key_presses_total[{step}s]))"
            ))
            .range(start, end, step)
            .send::<RangeResponse>()
            .await
    }
}
