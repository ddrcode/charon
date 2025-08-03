use std::sync::Arc;

use charon_lib::event::{DomainEvent, Mode};
use tokio::{
    fs::{OpenOptions, read_to_string},
    io::AsyncWriteExt,
};
use tracing::error;

use crate::domain::{
    AppEvent, Command, Context,
    traits::{ExternalApp, UiApp},
};

pub struct Password {
    ctx: Arc<Context>,
    should_exit: bool,
}

impl Password {
    pub fn new_box(ctx: Arc<Context>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Password {
            ctx,
            should_exit: false,
        })
    }

    async fn read_password(&self) -> std::io::Result<String> {
        read_to_string(&self.ctx.config.clipboard_cache_file).await
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

    fn path_to_app(&self) -> String {
        String::from("passepartui")
    }

    async fn on_start(&mut self) -> eyre::Result<()> {
        self.should_exit = false;
        self.clear_cache().await?;
        Ok(())
    }

    async fn process_result(&mut self, _out: &std::process::Output) -> Option<Command> {
        let Ok(pwd) = self.read_password().await else {
            self.should_exit = true;
            error!("Couldn't read password");
            return None;
        };

        Some(Command::SendEvent(DomainEvent::SendText(pwd)))
    }

    async fn handle_event(&mut self, event: &AppEvent) -> Option<Command> {
        if matches!(event, AppEvent::Backend(DomainEvent::TextSent)) {
            self.should_exit = true;
            return Some(Command::SendEvent(DomainEvent::ModeChange(
                Mode::PassThrough,
            )));
        }
        None
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }
}
