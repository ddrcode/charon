use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{InputConfig, defaults};
use crate::{config::keyboard::KeyboardConfig, domain::KeyShortcut};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharonConfig {
    #[serde(default)]
    pub keyboard: InputConfig,

    #[serde(default = "defaults::default_hid_keyboard")]
    pub hid_keyboard: PathBuf,

    #[serde(default = "defaults::default_typing_interval")]
    pub typing_interval: u8,

    #[serde(default = "defaults::default_server_socket")]
    pub server_socket: PathBuf,

    #[serde(default = "defaults::default_channel_size")]
    pub channel_size: usize,

    #[serde(with = "shortcut")]
    #[serde(default = "defaults::default_quit_shortcut")]
    pub quit_shortcut: KeyShortcut,

    #[serde(with = "shortcut")]
    #[serde(default = "defaults::default_toggle_mode_shortcut")]
    pub toggle_mode_shortcut: KeyShortcut,

    #[serde(with = "shortcut")]
    #[serde(default = "defaults::default_awake_host_shortcut")]
    pub awake_host_shortcut: KeyShortcut,

    #[serde(default)]
    pub host_mac_address: Option<Vec<u8>>,

    #[serde(default)]
    pub enable_telemetry: bool,

    #[serde(default)]
    pub keyboards: Option<KeyboardConfig>,

    #[serde(default = "defaults::default_time_to_sleep")]
    pub time_to_sleep: u64,

    #[serde(default)]
    pub sleep_script: Option<PathBuf>,

    #[serde(default)]
    pub awake_script: Option<PathBuf>,

    #[serde(default = "defaults::default_stats_file")]
    pub stats_file: PathBuf,

    #[serde(default = "defaults::default_stats_save_interval")]
    pub stats_save_interval: u64,

    #[serde(default = "defaults::default_stats_wpm_slot_duration")]
    pub stats_wpm_slot_duration: u64,

    #[serde(default = "defaults::default_stats_wpm_slot_count")]
    pub stats_wpm_slot_count: usize,

    #[serde(default = "defaults::default_keymaps_dir")]
    pub keymaps_dir: String,

    #[serde(default = "defaults::default_host_keymap")]
    pub host_keymap: String,
}

impl CharonConfig {
    pub fn get_config_per_keyboard(&self) -> Vec<(String, CharonConfig)> {
        match &self.keyboard {
            InputConfig::Use(alias) => self
                .keyboards
                .as_ref()
                .and_then(|k| k.groups.get(&alias.to_string()))
                .map(|group| {
                    group
                        .devices
                        .iter()
                        .map(|dev| {
                            let mut config = self.clone();
                            config.keyboards = None;
                            config.keyboard = InputConfig::Name(dev.name.clone().into());
                            (dev.alias.clone(), config)
                        })
                        .collect()
                })
                .unwrap_or_default(),
            _ => {
                let config = self.clone();
                let name = String::from("KeyScanner");
                vec![(name, config)]
            }
        }
    }
}

impl Default for CharonConfig {
    fn default() -> Self {
        Self {
            keyboard: InputConfig::default(),
            hid_keyboard: defaults::default_hid_keyboard(),
            typing_interval: defaults::default_typing_interval(),
            server_socket: defaults::default_server_socket(),
            channel_size: defaults::default_channel_size(),
            quit_shortcut: defaults::default_quit_shortcut(),
            toggle_mode_shortcut: defaults::default_toggle_mode_shortcut(),
            awake_host_shortcut: defaults::default_awake_host_shortcut(),
            host_mac_address: None,
            enable_telemetry: false,
            keyboards: None,
            time_to_sleep: defaults::default_time_to_sleep(),
            sleep_script: None,
            awake_script: None,
            stats_file: defaults::default_stats_file(),
            stats_save_interval: defaults::default_stats_save_interval(),
            stats_wpm_slot_duration: defaults::default_stats_wpm_slot_duration(),
            stats_wpm_slot_count: defaults::default_stats_wpm_slot_count(),
            keymaps_dir: defaults::default_keymaps_dir(),
            host_keymap: defaults::default_host_keymap(),
        }
    }
}

mod shortcut {
    use std::str::FromStr;

    use serde::Deserialize;
    use serde::de::{self, Deserializer};
    use serde::ser::Serializer;

    use crate::domain::KeyShortcut;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<KeyShortcut, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        KeyShortcut::from_str(&s).map_err(de::Error::custom)
    }

    pub fn serialize<S>(value: &KeyShortcut, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use toml;

    #[test]
    fn serialize() {
        let config = CharonConfig {
            keyboard: InputConfig::OneOf(vec!["keyb-1".into(), "keyb-2".into()]),
            ..Default::default()
        };
        let s = toml::to_string(&config);
        assert!(s.is_ok());
    }
}
