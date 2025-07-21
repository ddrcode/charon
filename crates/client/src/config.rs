use std::time::Duration;

pub struct AppConfig {
    pub idle_time: Duration,
    pub wisdom_duration: Duration,
    pub splash_duration: Duration,
    pub fast_typing_treshold: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            idle_time: Duration::from_secs(30),
            wisdom_duration: Duration::from_secs(12),
            splash_duration: Duration::from_secs(18),
            fast_typing_treshold: 35,
        }
    }
}
