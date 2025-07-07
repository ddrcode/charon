use std::fs::remove_file;
use std::path::Path;

use crate::domain::{Actor, DomainEvent, Event};
use tokio::net::UnixListener;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::info;

use super::{ClientSession, ClientSessionState};

pub struct IPCServer {
    listener: UnixListener,
    broker_tx: Sender<Event>,
    broker_rx: Receiver<Event>,
    session: Option<ClientSessionState>,
    alive: bool,
}

impl IPCServer {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        let path = "/tmp/charon.sock";
        if Path::new(path).exists() {
            remove_file(path).unwrap();
        }
        let listener = UnixListener::bind(path).unwrap();
        Self {
            broker_tx: tx,
            broker_rx: rx,
            session: None,
            listener,
            alive: true,
        }
    }

    async fn handle_event(&mut self, event: Event) {
        if let Some(session) = &self.session {
            if let Err(e) = session.sender.send(event.clone()).await {
                tracing::warn!("Failed to send event to session: {e}");
            }
        }
        match &event.payload {
            DomainEvent::Exit => self.alive = false,
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl Actor for IPCServer {
    async fn run(&mut self) {
        info!("Starting ipc-server");
        while self.alive {
            tokio::select! {
                // Accept a new connection
                Ok((stream, _)) = self.listener.accept() => {
                    tracing::info!("Accepted new IPC client");
                    // if let Some(old) = self.session.take() {
                    //     tracing::warn!("Replacing existing session");
                    //     old.shutdown().await;
                    // }

                    let (session_tx, session_rx) = mpsc::channel::<Event>(128);
                    let mut session = ClientSession::new(stream, self.broker_tx.clone(), session_rx);
                    let handle = tokio::spawn(async move { session.run().await; });
                    self.session = Some(ClientSessionState::new(handle, session_tx));

                }

                // Read broker events and push to session
                Some(event) = self.broker_rx.recv() => {
                    self.handle_event(event).await;
                }
            }
        }
    }

    fn id() -> &'static str {
        "ipc-server"
    }

    fn sender(&self) -> &Sender<Event> {
        &self.broker_tx
    }
}
