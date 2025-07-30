use std::{borrow::Cow, sync::Arc};

use charon_lib::event::{Event, Mode};
use tokio::sync::{
    RwLock,
    mpsc::{Receiver, Sender},
};

use crate::{config::CharonConfig, domain::traits::Processor};

pub struct ActorState {
    pub(crate) id: Cow<'static, str>,
    pub(crate) alive: bool,
    pub(crate) sender: Sender<Event>,
    pub(crate) receiver: Receiver<Event>,
    mode: Arc<RwLock<Mode>>,
    config: CharonConfig,
    processors: Vec<Box<dyn Processor + Send + Sync>>,
}

impl ActorState {
    pub fn new(
        id: Cow<'static, str>,
        mode: Arc<RwLock<Mode>>,
        sender: Sender<Event>,
        receiver: Receiver<Event>,
        config: CharonConfig,
        processors: Vec<Box<dyn Processor + Send + Sync>>,
    ) -> Self {
        Self {
            id,
            mode,
            alive: true,
            sender,
            receiver,
            config,
            processors,
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

    pub fn has_processors(&self) -> bool {
        !self.processors.is_empty()
    }

    pub fn iter_processors(
        &mut self,
    ) -> impl Iterator<Item = &mut Box<dyn Processor + Send + Sync>> {
        self.processors.iter_mut()
    }
}
