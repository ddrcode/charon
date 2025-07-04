use tokio::sync::mpsc::{Receiver, Sender};

use crate::domain::Event;

pub fn spawn_ipc_server(fx: Sender<Event>, tr: Receiver<Event>) {}
