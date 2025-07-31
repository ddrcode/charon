use std::sync::Arc;

use charon_lib::event::DomainEvent;
use tokio::task::spawn_blocking;
use tracing::error;

use crate::domain::{
    AppPhase, Command, Context,
    traits::{ExternalApp, UiApp},
};

pub struct Editor {
    _ctx: Arc<Context>,
    phase: AppPhase,
}

impl Editor {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self {
            _ctx: ctx,
            phase: AppPhase::default(),
        }
    }

    pub fn new_box(ctx: Arc<Context>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Editor::new(ctx))
    }

    async fn edit(&mut self) -> anyhow::Result<String> {
        use tempfile::NamedTempFile;

        let tmp = NamedTempFile::new()?;
        let path = tmp.into_temp_path().keep()?; // closes handle, keeps file alive
        let path_for_child = path.to_path_buf();

        spawn_blocking(move || {
            std::process::Command::new("nvim")
                .arg(&path_for_child)
                .status()
        })
        .await??;

        let path = path.to_string_lossy().to_string();
        Ok(path)
    }
}

#[async_trait::async_trait]
impl ExternalApp for Editor {
    fn id(&self) -> &'static str {
        "editor"
    }

    fn phase(&self) -> AppPhase {
        self.phase
    }

    fn set_phase(&mut self, phase: AppPhase) {
        self.phase = phase;
    }

    async fn run(&mut self) -> Option<Command> {
        let cmd = match self.edit().await {
            Ok(path) => Command::SendEvent(DomainEvent::SendFile(path, true)),
            Err(err) => {
                error!("Error running editor: {err}");
                return None;
            }
        };
        Some(cmd)
    }
}
