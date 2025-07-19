use ratatui::Frame;

use crate::domain::AppMsg;

#[async_trait::async_trait]
pub trait UiApp: Send + Sync {
    fn id(&self) -> &'static str;
    async fn update(&mut self, event: &AppMsg); // -> Option<ClientCommand>;
    async fn start(&mut self) {} // default: no-op
    async fn stop(&mut self) {}
    fn render(&self, frame: &mut Frame);
}
