use std::path::PathBuf;

use charon_lib::event::CharonEvent;
use maiko::{Context, Runtime};
use tokio::{process::Command, select};
use tracing::{error, info, warn};

use crate::domain::ActorState;

pub struct PowerManager {
    ctx: Context<CharonEvent>,
    state: ActorState,
    asleep: bool,
}

impl PowerManager {
    pub fn new(ctx: Context<CharonEvent>, state: ActorState) -> Self {
        Self {
            ctx,
            state,
            asleep: false,
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

    async fn handle_event(&mut self, event: &Self::Event) -> maiko::Result<()> {
        match event {
            CharonEvent::Exit => self.ctx.stop(),
            CharonEvent::KeyPress(..) if self.asleep => self.handle_awake().await?,
            _ => {}
        }
        Ok(())
    }

    async fn tick(&mut self, runtime: &mut Runtime<'_, Self::Event>) -> maiko::Result {
        let time_to_sleep = self.state.config().time_to_sleep;
        let time_to_sleep = tokio::time::Duration::from_secs(time_to_sleep);

        select! {
            Some(ref envelope) = runtime.recv() => {
                runtime.default_handle(self, envelope).await?;
            }
            _ = tokio::time::sleep(time_to_sleep) => {
                self.handle_sleep().await?;
            }
        }
        Ok(())
    }
}
