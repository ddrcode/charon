// SPDX-License-Identifier: GPL-3.0-or-later
use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, path::PathBuf, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub daemon_socket: PathBuf,

    pub idle_time: Duration,
    pub wisdom_duration: Duration,
    pub splash_duration: Duration,
    pub fast_typing_treshold: u16,

    pub clipboard_cache_file: PathBuf,
    pub keyboard_layout_file: PathBuf,
    pub keymap_file: PathBuf,

    pub password_app: String,
    pub editor_app: String,
}

impl AppConfig {
    pub fn from_file() -> eyre::Result<Self> {
        let mut path = PathBuf::new();
        path.push(std::env::var("XDG_CONFIG_HOME")?);
        path.push("charon/tui.toml");

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
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut clip_cache = PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap());
        clip_cache.push("charon/clipboard-cache");

        Self {
            daemon_socket: PathBuf::from("/tmp/charon.sock"),
            idle_time: Duration::from_secs(300),
            wisdom_duration: Duration::from_secs(60),
            splash_duration: Duration::from_secs(180),
            fast_typing_treshold: 40,

            keyboard_layout_file: PathBuf::from(format!(
                "{}/data/layouts/keychron_q10_ansi.txt",
                env!("CARGO_MANIFEST_DIR")
            )),
            keymap_file: PathBuf::from(format!(
                "{}/data/keymaps/keychron_10_ansi.json",
                env!("CARGO_MANIFEST_DIR")
            )),
            clipboard_cache_file: clip_cache,

            password_app: "passepartui".into(),
            editor_app: "nvim".into(),
        }
    }
}
