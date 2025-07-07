use std::sync::Arc;

use charon_lib::domain::{Event, Mode};
use tokio::sync::{
    RwLock,
    mpsc::{Receiver, Sender},
};

pub struct ActorState {
    pub(crate) id: &'static str,
    pub(crate) alive: bool,
    pub(crate) sender: Sender<Event>,
    pub(crate) receiver: Receiver<Event>,
    mode: Arc<RwLock<Mode>>,
}

impl ActorState {
    pub fn new(
        id: &'static str,
        mode: Arc<RwLock<Mode>>,
        sender: Sender<Event>,
        receiver: Receiver<Event>,
    ) -> Self {
        Self {
            id,
            mode,
            alive: true,
            sender,
            receiver,
        }
    }

    pub async fn mode(&self) -> Mode {
        *self.mode.read().await
    }

    pub async fn set_mode(&mut self, mode: Mode) {
        *self.mode.write().await = mode;
    }
}
