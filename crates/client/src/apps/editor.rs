use charon_lib::event::DomainEvent;
use ratatui::Frame;
use tempfile::NamedTempFile;
use tokio::task::spawn_blocking;

use crate::{
    domain::{AppMsg, Command, traits::UiApp},
    tui::{resume_tui, suspend_tui},
};

pub struct Editor {}

impl Editor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run() -> anyhow::Result<()> {
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_owned();

        Command::new("nvim").arg(&path).status()?;

        Ok(())
    }
}

impl UiApp for Editor {
    async fn start(&mut self) -> anyhow::Result<()> {
        use std::process::Command;
        use tempfile::NamedTempFile;

        let tmp = NamedTempFile::new()?;
        let path = tmp.into_temp_path().keep()?; // closes handle, keeps file alive
        let path_for_child = path.to_path_buf();

        suspend_tui(&mut self.terminal)?;
        spawn_blocking(move || Command::new("nvim").arg(&path_for_child).status()).await??;
        resume_tui(&mut self.terminal)?;

        self.terminal.clear()?;
        self.redraw()?;
        // self.switch_screen(Screen::Popup(
        //     "Please wait".into(),
        //     "Sending text...\nPress <[magic key]> to interrupt".into(),
        // ))?;

        let path = path.to_string_lossy().to_string();
        self.send(&DomainEvent::SendFile(path, true)).await?;

        Ok(())
    }

    async fn update(&mut self, msg: &AppMsg) -> Option<Command> {}
    fn render(&self, f: &mut Frame) {}
}
