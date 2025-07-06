use crate::domain::Event;
use tokio::net::UnixListener;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;

use super::{ClientSession, ClientSessionState};

pub struct IPCServer {
    listener: UnixListener,
    broker_tx: Sender<Event>,
    broker_rx: Receiver<Event>,
    session: Option<ClientSessionState>,
}

impl IPCServer {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        let listener = UnixListener::bind("/tmp/charon.sock").unwrap();
        Self {
            broker_tx: tx,
            broker_rx: rx,
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
                        // old.shutdown().await;
                    }

                    let (session_tx, session_rx) = mpsc::channel::<Event>(128);
                    let mut session = ClientSession::new(stream, self.broker_tx.clone(), session_rx);
                    let handle = tokio::spawn(async move { session.run(); });
                    self.session = Some(ClientSessionState::new(handle, session_tx));

                }

                // Read broker events and push to session
                Some(event) = self.broker_rx.recv() => {
                    if let Some(session) = &self.session {
                        if let Err(e) = session.sender.send(event).await {
                            tracing::warn!("Failed to send event to session: {e}");
                        }
                    }
                }
            }
        }
    }
}
