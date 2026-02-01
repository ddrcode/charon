// SPDX-License-Identifier: GPL-3.0-or-later
use super::DeviceEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardGroup {
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,

    #[serde(default)]
    pub raw_hid_enabled: bool,

    pub devices: Vec<DeviceEntry>,
}
