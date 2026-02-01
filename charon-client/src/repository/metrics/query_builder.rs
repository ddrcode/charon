// SPDX-License-Identifier: GPL-3.0-or-later
use reqwest::Client;
use tracing::debug;

pub struct QueryBuilder<'a> {
    client: &'a Client,
    base_url: &'a str,
    query: String,
    start: u64,
    end: u64,
    step: u64,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(client: &'a Client, base_url: &'a str) -> Self {
        Self {
            client,
            base_url,
            query: String::new(),
            start: 0,
            end: 0,
            step: 1,
        }
    }

    pub fn query(mut self, q: impl Into<String>) -> Self {
        self.query = q.into();
        self
    }

    pub fn range(mut self, start: u64, end: u64, step: u64) -> Self {
        self.start = start;
        self.end = end;
        self.step = step;
        self
    }

    pub async fn send<T: serde::de::DeserializeOwned>(self) -> eyre::Result<T> {
        let req = self
            .client
            .get(format!("{}/query_range", self.base_url))
            .query(&[
                ("query", self.query),
                ("start", self.start.to_string()),
                ("end", self.end.to_string()),
                ("step", format!("{}s", self.step)),
            ])
            .build()?;
        debug!("Sending query: {:?}", req.url().query());
        let resp = self.client.execute(req).await?.error_for_status()?;
        Ok(resp.json::<T>().await?)
    }
}
