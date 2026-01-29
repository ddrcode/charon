use std::sync::Arc;

use crate::domain::CharonEvent;
use maiko::Envelope;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

pub struct ClientSessionState {
    pub handle: JoinHandle<()>,
    pub sender: Sender<Arc<Envelope<CharonEvent>>>,
}

impl ClientSessionState {
    pub fn new(handle: JoinHandle<()>, sender: Sender<Arc<Envelope<CharonEvent>>>) -> Self {
        Self { handle, sender }
    }
}
