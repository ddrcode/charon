use std::collections::HashMap;

use charon_lib::event::{DomainEvent, Mode};
use ratatui::Frame;

use crate::domain::{AppMsg, Command, traits::UiApp};

pub struct AppManager {
    apps: HashMap<&'static str, Box<dyn UiApp + Send + Sync>>,
    active_id: &'static str,
}

impl AppManager {
    pub fn new(
        apps: HashMap<&'static str, Box<dyn UiApp + Send + Sync>>,
        active_id: &'static str,
    ) -> Self {
        Self { apps, active_id }
    }

    pub fn render(&self, frame: &mut Frame) {
        if let Some(app) = self.apps.get(&self.active_id) {
            app.render(frame);
        }
    }

    pub async fn update(&mut self, msg: &AppMsg) -> Option<Command> {
        match msg {
            AppMsg::Backend(DomainEvent::ModeChange(mode)) => {
                self.active_id = Self::mode_screen(mode);
                Some(Command::Render)
            }
            m => {
                if let Some(app) = self.apps.get_mut(&self.active_id) {
                    app.update(m).await
                } else {
                    None
                }
            }
        }
    }

    fn mode_screen(mode: &Mode) -> &'static str {
        match mode {
            Mode::PassThrough => "charonsay",
            Mode::InApp => "menu",
        }
    }
}
