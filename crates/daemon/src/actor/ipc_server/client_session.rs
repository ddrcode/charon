use charon_lib::event::{DomainEvent, Event};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::WriteHalf;
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
        let mut alive = true;

        while alive {
            tokio::select! {
                Ok(n) = reader.read_line(&mut line) => {
                    if n == 0 {
                        break;
                    }
                    Self::handle_stream(&line, self.broker_tx.clone()).await;
                    line.clear();
                }
                Some(event) = self.session_rx.recv() => {
                    alive = Self::handle_event(&event, &mut writer).await;
                }
            }
        }
    }

    async fn handle_stream(line: &str, broker_tx: Sender<Event>) {
        info!("Received: {}", line.trim());
        let event = serde_json::from_str::<Event>(line).unwrap();
        if let Err(e) = broker_tx.send(event).await {
            tracing::warn!("Failed to send to broker: {e}");
        }
    }

    async fn handle_event(event: &Event, writer: &mut WriteHalf<'_>) -> bool {
        let payload = serde_json::to_string(event).unwrap();

        writer.write_all(payload.as_bytes()).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
        // writer.flush().await.unwrap();

        match &event.payload {
            DomainEvent::Exit => return false,
            _ => {}
        }
        true
    }

    pub async fn send(&mut self, event: Event) -> anyhow::Result<()> {
        let stream = &mut self.stream;
        let payload = serde_json::to_string(&event)?;
        stream.write_all(payload.as_bytes()).await?;
        Ok(())
    }
}
