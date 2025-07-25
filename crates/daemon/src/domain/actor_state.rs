use std::{borrow::Cow, sync::Arc};

use charon_lib::event::{Event, Mode};
use tokio::sync::{
    RwLock,
    mpsc::{Receiver, Sender},
};

use crate::config::CharonConfig;

pub struct ActorState {
    pub(crate) id: Cow<'static, str>,
    pub(crate) alive: bool,
    pub(crate) sender: Sender<Event>,
    pub(crate) receiver: Receiver<Event>,
    mode: Arc<RwLock<Mode>>,
    config: CharonConfig,
}

impl ActorState {
    pub fn new(
        id: Cow<'static, str>,
        mode: Arc<RwLock<Mode>>,
        sender: Sender<Event>,
        receiver: Receiver<Event>,
        config: CharonConfig,
    ) -> Self {
        Self {
            id,
            mode,
            alive: true,
            sender,
            receiver,
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
