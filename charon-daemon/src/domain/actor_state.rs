use std::sync::Arc;

use super::Mode;
use tokio::sync::RwLock;

use crate::config::CharonConfig;

#[derive(Clone)]
pub struct ActorState {
    mode: Arc<RwLock<Mode>>,
    config: Arc<CharonConfig>,
}

impl ActorState {
    pub fn new(mode: Mode, config: Arc<CharonConfig>) -> Self {
        Self {
            mode: Arc::new(RwLock::new(mode)),
            config,
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

    pub fn clone_mode(&self) -> Arc<RwLock<Mode>> {
        self.mode.clone()
    }
}
