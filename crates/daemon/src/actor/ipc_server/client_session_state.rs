use tokio::{sync::mpsc::Sender, task::JoinHandle};

use crate::domain::Event;

pub struct ClientSessionState {
    pub handle: JoinHandle<()>,
    pub sender: Sender<Event>,
}

impl ClientSessionState {
    pub fn new(handle: JoinHandle<()>, sender: Sender<Event>) -> Self {
        Self { handle, sender }
    }
}
