use charon_lib::domain::{DomainEvent, Event, Mode};
use evdev::KeyCode;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::{
    actor::passthrough::Typist,
    devices::HIDKeyboard,
    domain::{Actor, ActorState, HidKeyCode, KeyboardState, Modifiers},
};

pub struct PassThrough {
    state: ActorState,
    report: KeyboardState,
    hidg: HIDKeyboard,
}

impl PassThrough {
    pub fn new(state: ActorState) -> Self {
        Self {
            state,
            report: KeyboardState::new(),
            hidg: HIDKeyboard::new("/dev/hidg0"),
        }
    }

    async fn handle_key_press(&mut self, key: &KeyCode) {
        let key = HidKeyCode::try_from(key).unwrap();
        self.report.update_on_press(key);
        if self
            .report
            .is(HidKeyCode::KEY_CAPSLOCK, Modifiers::default())
        {
            self.toggle_mode().await;
        } else if self.report.is(HidKeyCode::KEY_Q, Modifiers::LEFT_CTRL) {
            self.send(DomainEvent::Exit).await;
        } else {
            self.send_report().await;
        }
    }

    async fn handle_key_release(&mut self, key: &KeyCode) {
        let key = HidKeyCode::try_from(key).unwrap();
        self.report.update_on_release(key);
        self.send_report().await;
    }

    async fn toggle_mode(&mut self) {
        self.reset();
        let mode = self.state.mode().await.toggle();
        debug!("Switching mode to {:?}", mode);
        self.state.set_mode(mode).await;
        self.send(DomainEvent::ModeChange(mode)).await;
    }

    async fn send_report(&mut self) {
        if self.state.mode().await == Mode::PassThrough {
            let report = self.report.to_report();
            self.hidg.send_report(&report);
        }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::KeyPress(key) => {
                self.handle_key_press(key).await;
            }
            DomainEvent::KeyRelease(key) => {
                self.handle_key_release(key).await;
            }
            DomainEvent::SendFile(path) => {
                self.handle_file_send(path).await;
            }
            DomainEvent::Exit => {
                self.stop().await;
            }
            e => {
                warn!("Unhandled event: {:?}", e);
            }
        }
    }

    async fn handle_file_send(&mut self, path: &String) {
        let mut typist = Typist::default();
        typist.send_file(path, &mut self.hidg).await.unwrap();
    }

    pub fn reset(&mut self) {
        self.report.reset();
        self.hidg.reset();
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
