use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::KeyboardGroup;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardConfig {
    #[serde(flatten)]
    pub groups: HashMap<String, KeyboardGroup>,
}
