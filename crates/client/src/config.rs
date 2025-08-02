use std::{path::PathBuf, time::Duration};

pub struct AppConfig {
    pub daemon_socket: PathBuf,

    pub idle_time: Duration,
    pub wisdom_duration: Duration,
    pub splash_duration: Duration,
    pub fast_typing_treshold: u16,

    pub clipboard_cache_file: PathBuf,
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
            fast_typing_treshold: 35,

            clipboard_cache_file: clip_cache,
        }
    }
}
