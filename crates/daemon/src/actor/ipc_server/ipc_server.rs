use std::fs::remove_file;
use std::path::Path;

use crate::domain::{Actor, ActorState};
use charon_lib::domain::{DomainEvent, Event};
use tokio::sync::mpsc;
use tokio::{net::UnixListener, task::JoinHandle};
use tracing::info;

use super::{ClientSession, ClientSessionState};

pub struct IPCServer {
    state: ActorState,
    listener: UnixListener,
    session: Option<ClientSessionState>,
}

impl IPCServer {
    pub fn new(state: ActorState) -> Self {
        let path = "/tmp/charon.sock";
        if Path::new(path).exists() {
            remove_file(path).unwrap();
        }
        let listener = UnixListener::bind(path).unwrap();

        Self {
            state,
            session: None,
            listener,
        }
    }

    async fn handle_event(&mut self, event: Event) {
        if let Some(session) = &self.session {
            if let Err(e) = session.sender.send(event.clone()).await {
                tracing::warn!("Failed to send event to session: {e}");
            }
        }
        match &event.payload {
            DomainEvent::Exit => self.stop().await,
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl Actor for IPCServer {
    fn spawn(state: ActorState) -> JoinHandle<()> {
        let mut ipc_server = IPCServer::new(state);
        tokio::spawn(async move {
            ipc_server.run().await;
        })
    }

    async fn tick(&mut self) {
        tokio::select! {
            // Accept a new connection
            Ok((stream, _)) = self.listener.accept() => {
                info!("Accepted new IPC client");
                // if let Some(old) = self.session.take() {
                //     tracing::warn!("Replacing existing session");
                //     old.shutdown().await;
                // }

                let (session_tx, session_rx) = mpsc::channel::<Event>(128);
                let mut session = ClientSession::new(stream, self.state.sender.clone(), session_rx);
                let handle = tokio::spawn(async move { session.run().await; });
                self.session = Some(ClientSessionState::new(handle, session_tx));

            }

            // Read broker events and push to session
            Some(event) = self.state.receiver.recv() => {
                self.handle_event(event).await;
            }
        }
    }

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }

    fn filter(event: &Event) -> bool {
        if event.sender == "client" {
            return false;
        }
        match event.payload {
            DomainEvent::KeyPress(_) => false,
            DomainEvent::KeyRelease(_) => false,
            _ => true,
        }
    }
}
