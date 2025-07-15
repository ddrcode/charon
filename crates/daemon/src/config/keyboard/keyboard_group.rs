use super::DeviceEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardGroup {
    pub devices: Vec<DeviceEntry>,
}
