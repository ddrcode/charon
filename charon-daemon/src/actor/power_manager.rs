use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::domain::CharonEvent;
use maiko::{Context, Envelope, StepAction};
use tokio::process::Command;
use tracing::{error, info, warn};

use crate::domain::ActorState;

pub struct PowerManager {
    ctx: Context<CharonEvent>,
    state: ActorState,
    asleep: bool,
    last_event: Instant,
    time_to_sleep: Duration,
}

impl PowerManager {
    pub fn new(ctx: Context<CharonEvent>, state: ActorState) -> Self {
        let time_to_sleep = Duration::from_secs(state.config().time_to_sleep);
        Self {
            ctx,
            state,
            asleep: false,
            last_event: Instant::now(),
            time_to_sleep,
        }
    }

    async fn handle_sleep(&mut self) -> maiko::Result<()> {
        if let Some(path) = &self.state.config().sleep_script {
            if self.run_script(path.to_path_buf(), true).await {
                self.ctx.send(CharonEvent::Sleep).await?;
            }
        }
        Ok(())
    }

    async fn handle_awake(&mut self) -> maiko::Result<()> {
        self.last_event = Instant::now();
        if let Some(path) = &self.state.config().awake_script {
            if self.run_script(path.to_path_buf(), false).await {
                self.ctx.send(CharonEvent::WakeUp).await?;
            }
        }
        Ok(())
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

impl maiko::Actor for PowerManager {
    type Event = CharonEvent;

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> maiko::Result<()> {
        match envelope.event() {
            CharonEvent::KeyPress(..) if self.asleep => self.handle_awake().await?,
            CharonEvent::KeyPress(..) if !self.asleep => self.last_event = Instant::now(),
            _ => {}
        }
        Ok(())
    }

    async fn step(&mut self) -> maiko::Result<StepAction> {
        if self.last_event.elapsed() >= self.time_to_sleep {
            self.handle_sleep().await?;
            Ok(StepAction::AwaitEvent)
        } else {
            Ok(StepAction::Backoff(
                self.time_to_sleep - self.last_event.elapsed(),
            ))
        }
    }
}
