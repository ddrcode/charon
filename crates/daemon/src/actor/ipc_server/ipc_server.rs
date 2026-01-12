use std::path::Path;
use std::time::Duration;
use std::{fs, sync::Arc};

use crate::domain::ActorState;
use charon_lib::event::CharonEvent;
use maiko::{Context, Envelope, Meta, StepAction};
use tokio::net::UnixListener;
use tokio::sync::mpsc;
use tracing::info;

use super::{ClientSession, ClientSessionState};

pub struct IPCServer {
    ctx: Context<CharonEvent>,
    state: ActorState,
    listener: UnixListener,
    session: Option<ClientSessionState>,
}

impl IPCServer {
    pub fn new(ctx: Context<CharonEvent>, state: ActorState) -> Self {
        let path = state.config().server_socket.clone();
        if Path::new(&path).exists() {
            fs::remove_file(&path).expect("Couldn't remove socket file");
        }
        let listener = UnixListener::bind(path).expect("Couldn't create a socket file");

        Self {
            ctx,
            state,
            session: None,
            listener,
        }
    }
}

impl maiko::Actor for IPCServer {
    type Event = CharonEvent;

    async fn handle_envelope(&mut self, lope: &Arc<Envelope<Self::Event>>) -> maiko::Result {
        if let Some(session) = &self.session {
            if let Err(e) = session.sender.send(lope.clone()).await {
                tracing::warn!("Failed to send event to session: {e}");
                self.session = None;
            }
        }
        match &lope.event {
            // FIXME change dependency on actor name
            CharonEvent::ModeChange(mode) if lope.meta.actor_name() == "client" => {
                info!("Client requested to change mode to: {mode}");
                self.state.set_mode(*mode).await;
            }
            CharonEvent::Exit => self.ctx.stop(),
            _ => {}
        }
        Ok(())
    }

    async fn step(&mut self) -> maiko::Result<StepAction> {
        // Accept a new connection
        if let Ok((stream, _)) = self.listener.accept().await {
            info!("Accepted new IPC client");
            // if let Some(old) = self.session.take() {
            //     tracing::warn!("Replacing existing session");
            //     old.shutdown().await;
            // }

            let channel_size = self.state.config().channel_size;
            let mode = self.state.mode().await;
            let (session_tx, session_rx) =
                mpsc::channel::<Arc<Envelope<CharonEvent>>>(channel_size);
            let mut session = ClientSession::new(stream, self.ctx.clone(), session_rx);
            let handle = tokio::spawn(async move {
                session.init(mode).await;
                session.run().await;
            });
            self.session = Some(ClientSessionState::new(handle, session_tx));
        }
        Ok(StepAction::Backoff(Duration::from_millis(1000)))
    }
}
