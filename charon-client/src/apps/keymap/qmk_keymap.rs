use serde::Deserialize;
use std::path::Path;
use tokio::fs::read_to_string;

#[derive(Debug, Deserialize)]
pub struct QmkKeymap {
    pub keyboard: String,
    pub keymap: String,
    pub layout: String,
    pub layers: Vec<Vec<String>>,
}

impl QmkKeymap {
    pub async fn load(path: &Path) -> eyre::Result<Self> {
        let content = read_to_string(path).await?;
        let keymap: QmkKeymap = serde_json::from_str(&content)?;
        Ok(keymap)
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn layer(&self, n: usize) -> Option<&[String]> {
        self.layers.get(n).map(|v| v.as_slice())
    }
}
