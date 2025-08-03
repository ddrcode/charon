use std::{borrow::Cow, path::PathBuf, sync::Arc};

use charon_lib::event::{DomainEvent, Mode};
use tempfile::NamedTempFile;

use crate::domain::{
    AppEvent, Command, Context,
    traits::{ExternalApp, UiApp},
};

pub struct Editor {
    ctx: Arc<Context>,
    path: PathBuf,
    should_exit: bool,
}

impl Editor {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self {
            ctx,
            path: PathBuf::new(),
            should_exit: false,
        }
    }

    pub fn new_box(ctx: Arc<Context>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Editor::new(ctx))
    }

    fn path_to_string(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

#[async_trait::async_trait]
impl ExternalApp for Editor {
    fn id(&self) -> &'static str {
        "editor"
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn path_to_app(&self) -> Cow<'static, str> {
        self.ctx.config.editor_app.into()
    }

    fn app_args(&self) -> Vec<String> {
        vec![String::from("+star"), self.path_to_string()]
    }

    async fn on_start(&mut self) -> eyre::Result<()> {
        self.should_exit = false;
        let tmp = NamedTempFile::new()?;
        self.path = tmp.into_temp_path().keep()?; // closes handle, keeps file alive
        Ok(())
    }

    async fn process_result(&mut self) -> Option<Command> {
        Some(Command::SendEvent(DomainEvent::SendFile(
            self.path_to_string(),
            true,
        )))
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
}
