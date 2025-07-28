use std::path::PathBuf;

use charon_lib::event::{DomainEvent, Event};
use tokio::{process::Command, task::JoinHandle};
use tracing::{error, info, warn};

use crate::{
    domain::{ActorState, traits::Actor},
    error::CharonError,
};

pub struct PowerManager {
    state: ActorState,
    asleep: bool,
}

impl PowerManager {
    pub fn new(state: ActorState) -> Self {
        Self {
            state,
            asleep: false,
        }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::Exit => self.stop().await,
            DomainEvent::KeyPress(..) if self.asleep => self.handle_awake().await,
            _ => {}
        }
    }

    async fn handle_sleep(&mut self) {
        if let Some(path) = &self.state.config().sleep_script {
            if self.run_script(path.to_path_buf(), true).await {
                self.send(DomainEvent::Sleep).await;
            }
        }
    }

    async fn handle_awake(&mut self) {
        if let Some(path) = &self.state.config().awake_script {
            if self.run_script(path.to_path_buf(), false).await {
                self.send(DomainEvent::WakeUp).await;
            }
        }
    }

    async fn run_script(&mut self, path: PathBuf, should_sleep: bool) -> bool {
        if self.asleep == should_sleep {
            return false;
        }
        match Command::new(path).status().await {
            Ok(status) => {
                if status.success() {
                    self.asleep = should_sleep;
                    info!(
                        "Charon is {}",
                        if should_sleep { "asleep" } else { "awake" }
                    );
                    true
                } else {
                    warn!(
                        "{} script failed",
                        if should_sleep { "Sleep" } else { "Awake" }
                    );
                    false
                }
            }
            Err(err) => {
                error!("Error while executing power script: {err}");
                false
            }
        }
    }
}

#[async_trait::async_trait]
impl Actor for PowerManager {
    type Init = ();

    fn name() -> &'static str {
        "PowerManager"
    }

    fn spawn(state: ActorState, (): ()) -> Result<JoinHandle<()>, CharonError> {
        let mut power_mngr = PowerManager::new(state);
        let handle = tokio::task::spawn(async move {
            power_mngr.run().await;
        });
        Ok(handle)
    }

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }

    async fn tick(&mut self) {
        let time_to_sleep = self.state.config().time_to_sleep;
        let time_to_sleep = tokio::time::Duration::from_secs(time_to_sleep);

        tokio::select! {
            Some(event) = self.state.receiver.recv() => {
                self.handle_event(&event).await;
            }
            _ = tokio::time::sleep(time_to_sleep) => {
                self.handle_sleep().await;
            }
        }
    }

    async fn shutdown(&mut self) {
        self.handle_awake().await;
    }
}
