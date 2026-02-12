// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use super::Mode;
use tokio::sync::watch;

use crate::config::CharonConfig;

#[derive(Clone)]
pub struct ActorState {
    mode_tx: watch::Sender<Mode>,
    mode_rx: watch::Receiver<Mode>,
    config: Arc<CharonConfig>,
}

impl ActorState {
    pub fn new(mode: Mode, config: Arc<CharonConfig>) -> Self {
        let (mode_tx, mode_rx) = watch::channel(mode);
        Self {
            mode_tx,
            mode_rx,
            config,
        }
    }

    pub fn mode(&self) -> Mode {
        *self.mode_rx.borrow()
    }

    pub fn set_mode(&self, mode: Mode) {
        let _ = self.mode_tx.send(mode);
    }

    pub fn mode_receiver(&self) -> watch::Receiver<Mode> {
        self.mode_rx.clone()
    }

    pub fn config(&self) -> &CharonConfig {
        &self.config
    }
}
