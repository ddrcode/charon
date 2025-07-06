use crate::domain::Event;
use tokio::net::UnixListener;
use tokio::sync::mpsc::{Receiver, Sender};

use super::ClientSession;

pub struct IPCServer {
    listener: UnixListener,
    broker_tx: Sender<Event>,
    ipc_rx: Receiver<Event>,
    session: Option<ClientSession>,
}

impl IPCServer {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        let listener = UnixListener::bind("/tmp/charon.sock").unwrap();
        Self {
            broker_tx: tx,
            ipc_rx: rx,
            session: None,
            listener,
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                // Accept a new connection
                Ok((stream, _)) = self.listener.accept() => {
                    tracing::info!("Accepted new IPC client");
                    if let Some(old) = self.session.take() {
                        tracing::warn!("Replacing existing session");
                        old.shutdown().await;
                    }
                    let session = ClientSession::new(stream, self.broker_tx.clone());
                    tokio::spawn(session.run());
                    self.session = Some(session);
                }

                // Read broker events and push to session
                Some(event) = self.ipc_rx.recv() => {
                    if let Some(session) = &self.session {
                        if let Err(e) = session.send(event).await {
                            tracing::warn!("Failed to send event to session: {e}");
                        }
                    }
                }
            }
        }
    }
}
