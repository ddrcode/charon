use tokio::sync::mpsc::{Receiver, Sender};

use crate::domain::Event;

pub struct ActorState {
    pub(crate) id: &'static str,
    pub(crate) alive: bool,
    pub(crate) sender: Sender<Event>,
    pub(crate) receiver: Receiver<Event>,
}

impl ActorState {
    pub fn new(id: &'static str, sender: Sender<Event>, receiver: Receiver<Event>) -> Self {
        Self {
            id,
            alive: true,
            sender,
            receiver,
        }
    }
}
