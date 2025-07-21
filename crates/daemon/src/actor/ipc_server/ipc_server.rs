use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::ActorState;
use crate::domain::traits::Actor;
use charon_lib::event::{DomainEvent, Event};
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
    pub fn new(state: ActorState, path: &PathBuf) -> Self {
        if Path::new(path).exists() {
            fs::remove_file(path).expect("Couldn't remove socket file");
        }
        let listener = UnixListener::bind(path).expect("Couldn't create a socket file");

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
                self.session = None;
            }
        }
        match &event.payload {
            DomainEvent::ModeChange(mode) if event.sender == "client" => {
                info!("Client requested to change mode to: {mode}");
                self.state.set_mode(*mode).await;
            }
            DomainEvent::Exit => self.stop().await,
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl Actor for IPCServer {
    type Init = ();

    fn name() -> &'static str {
        "IPCServer"
    }

    fn spawn(state: ActorState, (): ()) -> JoinHandle<()> {
        let path = state.config().server_socket.clone();
        let mut ipc_server = IPCServer::new(state, &path);
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

                let channel_size = self.state.config().channel_size;
                let mode = self.state.mode().await;
                let (session_tx, session_rx) = mpsc::channel::<Event>(channel_size);
                let mut session = ClientSession::new(stream, self.state.sender.clone(), session_rx);
                let handle = tokio::spawn(async move {
                    session.init(mode).await;
                    session.run().await;
                });
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
}
