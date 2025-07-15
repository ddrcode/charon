use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DeviceEntry {
    pub name: String,
    #[serde(default)]
    pub optional: bool,
}
