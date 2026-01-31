use std::sync::Arc;

use crate::domain::{CharonEvent, Mode};
use maiko::{Context, Envelope};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::WriteHalf;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct ClientSession {
    stream: UnixStream,
    ctx: Context<CharonEvent>,
    session_rx: Receiver<Arc<Envelope<CharonEvent>>>,
    cancel_token: Arc<CancellationToken>,
}

impl ClientSession {
    pub fn new(
        stream: UnixStream,
        ctx: Context<CharonEvent>,
        session_rx: Receiver<Arc<Envelope<CharonEvent>>>,
        cancel_token: Arc<CancellationToken>,
    ) -> Self {
        Self {
            stream,
            ctx,
            session_rx,
            cancel_token,
        }
    }

    pub async fn init(&mut self, mode: Mode) {
        self.send(CharonEvent::ModeChange(mode)).await.unwrap();
    }

    pub async fn run(&mut self) {
        let (reader, mut writer) = self.stream.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            tokio::select! {
                _ = self.cancel_token.cancelled() => break,
                Ok(n) = reader.read_line(&mut line) => {
                    if n == 0 {
                        break;
                    }
                    info!("Received: {}", line.trim());
                    let envelope = serde_json::from_str::<Envelope<CharonEvent>>(&line).unwrap();
                    if let Err(e) = self.ctx.send_envelope(envelope).await {
                        tracing::warn!("Failed to send to broker: {e}");
                    }
                    line.clear();
                }
                Some(event) = self.session_rx.recv() => {
                    Self::handle_event(&event, &mut writer).await;
                }
            }
        }
    }

    async fn handle_event(event: &Envelope<CharonEvent>, writer: &mut WriteHalf<'_>) {
        let payload = serde_json::to_string(event).unwrap();
        writer.write_all(payload.as_bytes()).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
        // writer.flush().await.unwrap();
    }

    pub async fn send(&mut self, event: CharonEvent) -> eyre::Result<()> {
        let stream = &mut self.stream;
        let event = Envelope::new(event, self.ctx.actor_id().clone());
        let payload = serde_json::to_string(&event)?;
        stream.write_all(payload.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        Ok(())
    }
}
