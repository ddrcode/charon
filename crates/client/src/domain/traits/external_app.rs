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
    fn path_to_app(&self) -> String;
    fn app_args(&self) -> Vec<String> {
        Vec::new()
    }

    async fn process_result(&mut self) -> Option<Command>;
    async fn on_start(&mut self) -> eyre::Result<()> {
        Ok(())
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
            AppEvent::ReturnFromExternal => self.process_result().await,
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
            "Please wait".into(),
            "Sending text...\nPress <[magic key]> to interrupt".into(),
        );
    }
}
