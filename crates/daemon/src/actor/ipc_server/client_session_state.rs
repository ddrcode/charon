use std::sync::Arc;

use charon_lib::event::DomainEvent;
use maiko::Envelope;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

pub struct ClientSessionState {
    pub handle: JoinHandle<()>,
    pub sender: Sender<Arc<Envelope<DomainEvent>>>,
}

impl ClientSessionState {
    pub fn new(handle: JoinHandle<()>, sender: Sender<Arc<Envelope<DomainEvent>>>) -> Self {
        Self { handle, sender }
    }
}
