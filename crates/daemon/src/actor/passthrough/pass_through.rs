use charon_lib::event::{DomainEvent, Event, Mode};
use evdev::KeyCode;
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::domain::{Actor, ActorState, HidKeyCode, KeyboardState, Modifiers};

pub struct PassThrough {
    state: ActorState,
    report: KeyboardState,
}

impl PassThrough {
    pub fn new(state: ActorState) -> Self {
        Self {
            state,
            report: KeyboardState::new(),
        }
    }

    async fn handle_key_press(&mut self, key: &KeyCode, source_id: &Uuid) {
        let key = match HidKeyCode::try_from(key) {
            Ok(val) => val,
            Err(e) => {
                return error!("{e}");
            }
        };
        self.report.update_on_press(key);
        if self.report.is(HidKeyCode::KEY_F7, Modifiers::default()) {
            self.toggle_mode().await;
        } else if self.report.is(HidKeyCode::KEY_Q, Modifiers::LEFT_CTRL) {
            self.send(DomainEvent::Exit).await;
        } else {
            self.send_report(source_id).await;
        }
    }

    async fn handle_key_release(&mut self, key: &KeyCode, source_id: &Uuid) {
        let key = match HidKeyCode::try_from(key) {
            Ok(val) => val,
            Err(e) => {
                return error!("{e}");
            }
        };
        self.report.update_on_release(key);
        self.send_report(source_id).await;
    }

    async fn toggle_mode(&mut self) {
        self.reset();
        let mode = self.state.mode().await.toggle();
        debug!("Switching mode to {:?}", mode);
        self.state.set_mode(mode).await;
        self.send(DomainEvent::ModeChange(mode)).await;
    }

    async fn send_report(&mut self, source_id: &Uuid) {
        if self.state.mode().await == Mode::PassThrough {
            let report = self.report.to_report();
            let payload = DomainEvent::HidReport(report);
            let event = Event::with_source_id(self.id(), payload, source_id.clone());
            self.send_raw(event).await;
        }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::KeyPress(key) => {
                self.handle_key_press(key, &event.id).await;
            }
            DomainEvent::KeyRelease(key) => {
                self.handle_key_release(key, &event.id).await;
            }
            DomainEvent::Exit => {
                self.stop().await;
            }
            e => {
                warn!("Unhandled event: {:?}", e);
            }
        }
    }

    pub fn reset(&mut self) {
        self.report.reset();
    }
}

impl Drop for PassThrough {
    fn drop(&mut self) {
        self.reset();
    }
}

#[async_trait::async_trait]
impl Actor for PassThrough {
    fn spawn(state: ActorState) -> JoinHandle<()> {
        let mut passthrough = PassThrough::new(state);
        tokio::spawn(async move {
            passthrough.run().await;
        })
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
