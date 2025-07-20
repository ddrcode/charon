use ratatui::Frame;

use crate::domain::{AppMsg, Command};

#[async_trait::async_trait]
pub trait UiApp: Send + Sync {
    fn id(&self) -> &'static str;
    async fn update(&mut self, event: &AppMsg) -> Option<Command>;
    fn render(&self, frame: &mut Frame);
}
