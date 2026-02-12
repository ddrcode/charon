// SPDX-License-Identifier: GPL-3.0-or-later
use std::{fs, path::Path, sync::Arc};

use maiko::{Context, Envelope, StepAction};
use tokio::{
    net::UnixListener,
    select,
    sync::{mpsc, watch},
};
use tokio_util::sync::CancellationToken;
use tracing::info;

use super::{ClientSession, ClientSessionState};
use crate::domain::{ActorState, CharonEvent, Mode};

pub struct IPCServer {
    ctx: Context<CharonEvent>,
    state: ActorState,
    mode_rx: watch::Receiver<Mode>,
    listener: UnixListener,
    session: Option<ClientSessionState>,
    cancel_token: Arc<CancellationToken>,
}

impl IPCServer {
    pub fn new(ctx: Context<CharonEvent>, state: ActorState) -> Self {
        let path = state.config().server_socket.clone();
        if Path::new(&path).exists() {
            fs::remove_file(&path).expect("Couldn't remove socket file");
        }
        let listener = UnixListener::bind(path).expect("Couldn't create a socket file");
        let mode_rx = state.mode_receiver();

        Self {
            ctx,
            state,
            mode_rx,
            session: None,
            listener,
            cancel_token: Arc::new(CancellationToken::new()),
        }
    }

    async fn send_to_client(&mut self, event: Envelope<CharonEvent>) {
        if let Some(session) = &self.session {
            if let Err(e) = session.sender.send(Arc::new(event)).await {
                tracing::warn!("Failed to send event to session: {e}");
                self.session = None;
            }
        }
    }
}

impl maiko::Actor for IPCServer {
    type Event = CharonEvent;

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> maiko::Result {
        if self.session.is_some() {
            self.send_to_client(envelope.clone()).await;
        }
        match envelope.event() {
            // FIXME change dependency on actor name
            CharonEvent::ModeChange(mode) if envelope.meta().actor_name() == "client" => {
                info!("Client requested to change mode to: {mode}");
                self.state.set_mode(*mode);
            }
            _ => {}
        }
        Ok(())
    }

    async fn step(&mut self) -> maiko::Result<StepAction> {
        select! {
            // Handle mode change
            Ok(()) = self.mode_rx.changed(), if self.session.is_some() => {
                let mode = *self.mode_rx.borrow_and_update();
                let event = CharonEvent::ModeChange(mode);
                self.send_to_client(Envelope::new(event, self.ctx.actor_id().clone())).await;
            }

            // Accept a new connection
            Ok((stream, _)) = self.listener.accept() => {
                info!("Accepted new IPC client");
                // if let Some(old) = self.session.take() {
                //     tracing::warn!("Replacing existing session");
                //     old.shutdown().await;
                // }

                let channel_size = self.state.config().channel_size;
                let mode = self.state.mode();
                let (session_tx, session_rx) =
                    mpsc::channel::<Arc<Envelope<CharonEvent>>>(channel_size);
                let mut session = ClientSession::new(
                    stream,
                    self.ctx.clone(),
                    session_rx,
                    self.cancel_token.clone(),
                );
                let handle = tokio::spawn(async move {
                    session.init(mode).await;
                    session.run().await;
                });
                self.session = Some(ClientSessionState::new(handle, session_tx));
            }
        }
        Ok(StepAction::Yield)
    }

    async fn on_shutdown(&mut self) -> maiko::Result<()> {
        self.cancel_token.cancel();
        Ok(())
    }
}
