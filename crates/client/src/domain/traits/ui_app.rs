use ratatui::Frame;

use crate::domain::AppMsg;

pub trait UiApp {
    fn id(&self) -> &'static str;
    fn update(&mut self, msg: &AppMsg);
    fn render(&self, frame: &mut Frame);
}
