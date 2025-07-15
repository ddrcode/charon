use super::DeviceEntry;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KeyboardGroup {
    pub devices: Vec<DeviceEntry>,
}
