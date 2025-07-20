use std::sync::Arc;

use charon_lib::event::DomainEvent;
use ratatui::Frame;
use tempfile::NamedTempFile;
use tokio::task::spawn_blocking;

use crate::{
    components::notification,
    domain::{AppMsg, Command, Context, traits::UiApp},
    tui::{resume_tui, suspend_tui},
};

pub struct Editor {
    ctx: Arc<Context>,
}

impl Editor {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub fn new_box(ctx: Arc<Context>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Self { ctx })
    }

    // pub fn run() -> anyhow::Result<()> {
    //     let tmp = NamedTempFile::new()?;
    //     let path = tmp.path().to_owned();
    //
    //     Command::new("nvim").arg(&path).status()?;
    //
    //     Ok(())
    // }
}

#[async_trait::async_trait]
impl UiApp for Editor {
    fn id(&self) -> &'static str {
        "editor"
    }

    // async fn start(&mut self) -> anyhow::Result<()> {
    //     use tempfile::NamedTempFile;
    //
    //     let tmp = NamedTempFile::new()?;
    //     let path = tmp.into_temp_path().keep()?; // closes handle, keeps file alive
    //     let path_for_child = path.to_path_buf();
    //
    //     suspend_tui(&mut self.terminal)?;
    //     spawn_blocking(move || {
    //         std::process::Command::new("nvim")
    //             .arg(&path_for_child)
    //             .status()
    //     })
    //     .await??;
    //     resume_tui(&mut self.terminal)?;
    //
    //     self.terminal.clear()?;
    //     self.redraw()?;
    //
    //     let path = path.to_string_lossy().to_string();
    //     self.send(&DomainEvent::SendFile(path, true)).await?;
    //
    //     Ok(())
    // }

    async fn update(&mut self, msg: &AppMsg) -> Option<Command> {
        match msg {
            _ => {}
        }
        None
    }

    fn render(&self, f: &mut Frame) {
        notification(
            f,
            "Please wait".into(),
            "Sending text...\nPress <[magic key]> to interrupt".into(),
        );
    }
}
