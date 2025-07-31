use std::sync::Arc;

use charon_lib::event::DomainEvent;
use tokio::{
    fs::{OpenOptions, read_to_string},
    io::AsyncWriteExt,
    task::spawn_blocking,
};
use tracing::error;

use crate::domain::{
    AppPhase, Command, Context,
    traits::{ExternalApp, UiApp},
};

pub struct Password {
    ctx: Arc<Context>,
    phase: AppPhase,
}

impl Password {
    pub fn new_box(ctx: Arc<Context>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Password {
            ctx,
            phase: AppPhase::default(),
        })
    }

    async fn run_external(&mut self) -> anyhow::Result<String> {
        spawn_blocking(|| std::process::Command::new("passepartui").status()).await??;
        let str = read_to_string(&self.ctx.config.clipboard_cache_file).await?;
        Ok(str)
    }

    async fn clear_cache(&self) -> std::io::Result<()> {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.ctx.config.clipboard_cache_file)
            .await?
            .write_all(b"")
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ExternalApp for Password {
    fn id(&self) -> &'static str {
        "password"
    }

    fn phase(&self) -> AppPhase {
        self.phase
    }

    fn set_phase(&mut self, phase: AppPhase) {
        self.phase = phase;
    }

    async fn run(&mut self) -> Option<Command> {
        let cmd = match self.run_external().await {
            Ok(pwd) => {
                if pwd.is_empty() {
                    return None;
                }
                Command::SendEvent(DomainEvent::SendText(pwd))
            }
            Err(err) => {
                error!("Error getting password: {err}");
                return None;
            }
        };
        Some(cmd)
    }

    async fn on_finish(&mut self) {
        if let Err(e) = self.clear_cache().await {
            error!("Failed clearing cache file: {e}");
        }
    }
}
