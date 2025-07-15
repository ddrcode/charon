use serde::Deserialize;
use std::collections::HashMap;

use super::KeyboardGroup;

#[derive(Debug, Deserialize)]
pub struct KeyboardConfig {
    #[serde(flatten)]
    pub groups: HashMap<String, KeyboardGroup>,
}
