use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

use super::IPCServer;
use crate::domain::{Actor, Event};

pub fn spawn_ipc_server(tx: Sender<Event>, rx: Receiver<Event>) -> JoinHandle<()> {
    let mut ipc_server = IPCServer::new(tx, rx);
    tokio::spawn(async move {
        ipc_server.run().await;
    })
}
