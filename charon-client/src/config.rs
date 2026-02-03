// SPDX-License-Identifier: GPL-3.0-or-later
use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, path::PathBuf, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "defaults::server_socket")]
    pub daemon_socket: PathBuf,

    #[serde(default = "defaults::idle_time")]
    pub idle_time: Duration,

    #[serde(default = "defaults::wisdom_duration")]
    pub wisdom_duration: Duration,

    #[serde(default = "defaults::splash_duration")]
    pub splash_duration: Duration,

    #[serde(default = "defaults::fast_typing_treshold")]
    pub fast_typing_treshold: u16,

    #[serde(default = "defaults::clipboard_cache")]
    pub clipboard_cache_file: PathBuf,

    #[serde(default = "defaults::keyboard_layout_file")]
    pub keyboard_layout_file: String,

    #[serde(default = "defaults::keymap_file")]
    pub keymap_file: String,

    #[serde(default = "defaults::password_app")]
    pub password_app: String,

    #[serde(default = "defaults::editor_app")]
    pub editor_app: String,

    #[serde(default = "defaults::layouts_dir")]
    pub keyboard_layouts_dir: PathBuf,

    #[serde(default = "defaults::keymaps_dir")]
    pub keymaps_dir: PathBuf,
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

    pub fn keyboard_layout_path(&self) -> PathBuf {
        let mut path = self.keyboard_layouts_dir.clone();
        path.push(&self.keyboard_layout_file);
        path
    }

    pub fn keymap_path(&self) -> PathBuf {
        let mut path = self.keymaps_dir.clone();
        path.push(&self.keymap_file);
        path
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            daemon_socket: defaults::server_socket(),
            idle_time: defaults::idle_time(),
            wisdom_duration: defaults::wisdom_duration(),
            splash_duration: defaults::splash_duration(),
            fast_typing_treshold: defaults::fast_typing_treshold(),

            keyboard_layout_file: defaults::keyboard_layout_file(),
            keymap_file: defaults::keymap_file(),
            keymaps_dir: defaults::keymaps_dir(),
            keyboard_layouts_dir: defaults::layouts_dir(),

            clipboard_cache_file: defaults::clipboard_cache(),

            password_app: defaults::password_app(),
            editor_app: defaults::editor_app(),
        }
    }
}

mod defaults {
    use std::path::PathBuf;
    use std::time::Duration;

    pub(super) fn idle_time() -> Duration {
        Duration::from_secs(300)
    }

    pub(super) fn wisdom_duration() -> Duration {
        Duration::from_secs(60)
    }

    pub(super) fn splash_duration() -> Duration {
        Duration::from_secs(180)
    }

    pub(super) fn fast_typing_treshold() -> u16 {
        40
    }

    pub(super) fn server_socket() -> PathBuf {
        PathBuf::from("/tmp/charon.sock")
    }

    pub(super) fn clipboard_cache() -> PathBuf {
        let mut path = PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap());
        path.push("charon/clipboard-cache");
        path
    }

    pub(super) fn layouts_dir() -> PathBuf {
        let mut path = PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("data");
        path.push("layouts");
        path
    }

    pub(super) fn keymaps_dir() -> PathBuf {
        let mut path = PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("data");
        path.push("keymaps");
        path
    }

    pub(super) fn keymap_file() -> String {
        "keychron_10_ansi.json".into()
    }

    pub(super) fn keyboard_layout_file() -> String {
        "keychron_10_ansi.txt".into()
    }

    pub(super) fn password_app() -> String {
        "passepartui".into()
    }

    pub(super) fn editor_app() -> String {
        "nvim".into()
    }
}
