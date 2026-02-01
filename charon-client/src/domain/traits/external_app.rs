// SPDX-License-Identifier: GPL-3.0-or-later
use std::{borrow::Cow, process::ExitStatus};

use ratatui::Frame;
use tracing::error;

use crate::{
    components::notification,
    domain::{AppEvent, Command},
};

use super::UiApp;

#[async_trait::async_trait]
pub trait ExternalApp {
    fn id(&self) -> &'static str;
    fn path_to_app(&self) -> Cow<'static, str>;
    fn app_args(&self) -> Vec<String> {
        Vec::new()
    }

    async fn process_result(&mut self) -> Option<Command>;
    async fn on_start(&mut self) -> eyre::Result<()> {
        Ok(())
    }
    async fn on_error(&mut self, _status: Option<&ExitStatus>) -> Option<Command> {
        Some(Command::ExitApp)
    }
    async fn handle_event(&mut self, _event: &AppEvent) -> Option<Command> {
        None
    }

    fn should_exit(&self) -> bool;
}

#[async_trait::async_trait]
impl<T> UiApp for T
where
    T: ExternalApp + Send + Sync,
{
    fn id(&self) -> &'static str {
        self.id()
    }

    async fn update(&mut self, msg: &AppEvent) -> Option<Command> {
        let cmd = match msg {
            AppEvent::Activate => {
                if let Err(err) = self.on_start().await {
                    error!("Couldn't activate external app: {err}");
                    Some(Command::ExitApp)
                } else {
                    Some(Command::RunExternal(self.path_to_app(), self.app_args()))
                }
            }
            AppEvent::ReturnFromExternal(status) => match status {
                Some(s) if s.success() => return self.process_result().await,
                _ => self.on_error(status.as_ref()).await,
            },
            _ => None,
        };

        if self.should_exit() {
            return Some(Command::ExitApp);
        }

        if cmd.is_some() {
            return cmd;
        }
        self.handle_event(msg).await
    }

    fn render(&self, f: &mut Frame) {
        notification(
            f,
            "Please wait",
            "Sending text...\nPress <[magic key]> to interrupt",
        );
    }
}
