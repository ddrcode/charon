use charon_lib::event::{DomainEvent, Event, Mode};
use tokio::{
    fs::{read_to_string, remove_file},
    task::JoinHandle,
};
use tracing::warn;

use crate::{
    domain::{Actor, ActorState, HidKeyCode, KeyboardState},
    error::KOSError,
};

pub struct Typist {
    state: ActorState,
    report: KeyboardState,
    speed: tokio::time::Duration,
}

impl Typist {
    pub fn new(state: ActorState, interval: u8) -> Self {
        Self {
            state,
            report: KeyboardState::new(),
            speed: tokio::time::Duration::from_millis(interval.into()),
        }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::SendText(txt) => self.send_string(txt).await,
            DomainEvent::SendFile(path, remove) => {
                self.send_file(path, *remove).await.expect("File not found")
            }
            DomainEvent::ModeChange(Mode::PassThrough) => {
                // cancel text sending
            }
            DomainEvent::Exit => self.stop().await,
            _ => {}
        }
    }

    pub async fn send_char(&mut self, c: char) {
        let seq = match HidKeyCode::seq_from_char(c) {
            Ok(val) => val,
            Err(_) => {
                warn!("Couldn't produce sequence for char {c}");
                return;
            }
        };
        for key in seq.iter() {
            self.report.update_on_press(*key);
            self.send(DomainEvent::HidReport(self.report.to_report()))
                .await;
            tokio::time::sleep(self.speed).await;
        }
        for key in seq.iter().rev() {
            self.report.update_on_release(*key);
            self.send(DomainEvent::HidReport(self.report.to_report()))
                .await;
            tokio::time::sleep(self.speed).await;
        }
    }

    pub async fn send_string(&mut self, s: &String) {
        for c in s.chars() {
            self.send_char(c).await;
            if self.state.mode().await == Mode::PassThrough {
                warn!("Sending text interrupted by mode change");
                return;
            }
        }
        self.send(DomainEvent::TextSent).await;
    }

    pub async fn send_file(&mut self, path: &String, remove: bool) -> Result<(), KOSError> {
        let text = read_to_string(path).await?;
        self.send_string(&text).await;
        if remove {
            remove_file(path).await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Actor for Typist {
    fn spawn(state: ActorState) -> JoinHandle<()> {
        let mut writer = Typist::new(state, 50);
        tokio::spawn(async move { writer.run().await })
    }

    async fn tick(&mut self) {
        if let Some(event) = self.recv().await {
            self.handle_event(&event).await;
        }
    }

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }
}
