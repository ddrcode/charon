use std::path::PathBuf;

use charon_lib::event::DomainEvent;
use maiko::{Context, Meta};
use tokio::process::Command;
use tracing::{error, info, warn};

use crate::domain::ActorState;

pub struct PowerManager {
    ctx: Context<DomainEvent>,
    state: ActorState,
    asleep: bool,
}

impl PowerManager {
    pub fn new(ctx: Context<DomainEvent>, state: ActorState) -> Self {
        Self {
            ctx,
            state,
            asleep: false,
        }
    }

    async fn handle_sleep(&mut self) -> maiko::Result<()> {
        if let Some(path) = &self.state.config().sleep_script {
            if self.run_script(path.to_path_buf(), true).await {
                self.ctx.send(DomainEvent::Sleep).await?;
            }
        }
        Ok(())
    }

    async fn handle_awake(&mut self) -> maiko::Result<()> {
        if let Some(path) = &self.state.config().awake_script {
            if self.run_script(path.to_path_buf(), false).await {
                self.ctx.send(DomainEvent::WakeUp).await?;
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
    type Event = DomainEvent;

    async fn handle(&mut self, event: &Self::Event, _meta: &Meta) -> maiko::Result<()> {
        match event {
            DomainEvent::Exit => self.ctx.stop(),
            DomainEvent::KeyPress(..) if self.asleep => self.handle_awake().await?,
            _ => {}
        }
        Ok(())
    }

    async fn tick(&mut self) -> maiko::Result<()> {
        let time_to_sleep = tokio::time::Duration::from_secs(self.state.config().time_to_sleep);
        tokio::time::sleep(time_to_sleep).await;
        self.handle_sleep().await?;
        Ok(())
    }
}
