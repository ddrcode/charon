use std::{borrow::Cow, sync::Arc, time::Instant};

use charon_lib::event::Mode;
use tokio::sync::RwLock;

use crate::config::CharonConfig;

#[derive(Debug, Clone)]
pub struct ProcessorState {
    pub(crate) id: Cow<'static, str>,
    mode: Arc<RwLock<Mode>>,
    config: CharonConfig,
    start_time: Instant,
}

impl ProcessorState {
    pub fn new(id: Cow<'static, str>, mode: Arc<RwLock<Mode>>, config: CharonConfig) -> Self {
        Self {
            id,
            mode,
            config,
            start_time: Instant::now(),
        }
    }

    pub async fn mode(&self) -> Mode {
        *self.mode.read().await
    }

    pub async fn set_mode(&mut self, mode: Mode) {
        *self.mode.write().await = mode;
    }

    pub fn config(&self) -> &CharonConfig {
        &self.config
    }

    pub fn start_time(&self) -> &Instant {
        &self.start_time
    }
}
