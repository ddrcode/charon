use crate::domain::Event;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;

#[derive(Debug)]
pub struct ClientSession {
    stream: UnixStream,
    broker_tx: Sender<Event>,
    session_rx: Receiver<Event>,
}

impl ClientSession {
    pub fn new(stream: UnixStream, broker_tx: Sender<Event>, session_rx: Receiver<Event>) -> Self {
        Self {
            stream,
            broker_tx,
            session_rx,
        }
    }

    pub async fn run(&mut self) {
        let (reader, mut writer) = self.stream.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            tokio::select! {
                Ok(n) = reader.read_line(&mut line) => {
                    if n == 0 {
                        tracing::info!("Client disconnected");
                        break;
                    }
                    self.handle_stream(&line);
                    line.clear();

                }
                Some(event) = self.session_rx.recv() => {
                    self.handle_event(&event).await;
                }
            }
        }
    }

    async fn handle_stream(&mut self, line: &str) {
        info!("Received: {}", line.trim());
        // Convert line → Event → send to broker
        // let event = parse_line_to_event(&line);
        // if let Err(e) = self.broker_tx.send(event).await {
        //     tracing::warn!("Failed to send to broker: {e}");
        //     break;
        // }
    }

    async fn handle_event(&mut self, event: &Event) {}

    pub async fn send(&mut self, event: Event) -> anyhow::Result<()> {
        let stream = &mut self.stream;
        let payload = serde_json::to_string(&event)?;
        stream.write_all(payload.as_bytes()).await?;
        Ok(())
    }

    pub async fn shutdown(self) {
        tracing::info!("Session shutting down");
        // Drop will close the stream automatically
    }
}
