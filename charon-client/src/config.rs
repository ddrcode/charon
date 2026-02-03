// SPDX-License-Identifier: GPL-3.0-or-later
use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, path::PathBuf, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub daemon_socket: PathBuf,
    pub idle_time: Duration,
    pub wisdom_duration: Duration,
    pub splash_duration: Duration,
    pub fast_typing_treshold: u16,
    pub clipboard_cache_file: PathBuf,
    pub keyboard_layout_file: String,
    pub keymap_file: String,
    pub password_app: String,
    pub editor_app: String,
    pub keyboard_layouts_dir: PathBuf,
    pub keymaps_dir: PathBuf,
    pub wisdoms_file: PathBuf,
    pub upgrade_script: PathBuf,
}

impl AppConfig {
    pub fn from_file(path: PathBuf) -> eyre::Result<Self> {
        if !path.exists() {
            tracing::warn!(
                "Couldn't find config file at {:?}. Starting with default configuration",
                path
            );
            return Ok(AppConfig::default());
        }

        tracing::debug!("Found config file: {:?}", path);
        let config_str = read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_str)?;

        Ok(config)
    }

    pub fn keyboard_layout_path(&self) -> PathBuf {
        self.keyboard_layouts_dir.join(&self.keyboard_layout_file)
    }

    pub fn keymap_path(&self) -> PathBuf {
        self.keymaps_dir.join(&self.keymap_file)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            daemon_socket: PathBuf::from("/tmp/charon.sock"),
            idle_time: Duration::from_secs(300),
            wisdom_duration: Duration::from_secs(60),
            splash_duration: Duration::from_secs(180),
            fast_typing_treshold: 40,
            clipboard_cache_file: PathBuf::from(
                std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into()),
            )
            .join("charon/clipboard-cache"),
            keyboard_layout_file: "keychron_10_ansi.txt".into(),
            keymap_file: "keychron_10_ansi.json".into(),
            keymaps_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/keymaps"),
            keyboard_layouts_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/layouts"),
            password_app: "passepartui".into(),
            editor_app: "nvim".into(),
            wisdoms_file: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/wisdoms.json"),
            upgrade_script: PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
                .join(".local/charon.service/upgrade.sh"),
        }
    }
}
