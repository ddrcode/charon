use tokio::task::JoinHandle;

use super::IPCServer;
use crate::domain::{Actor, ActorState};

pub fn spawn_ipc_server(state: ActorState) -> JoinHandle<()> {
    let mut ipc_server = IPCServer::new(state);
    tokio::spawn(async move {
        ipc_server.run().await;
    })
}
