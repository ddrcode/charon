use std::collections::HashMap;

use ratatui::Frame;

use crate::domain::{AppMsg, traits::UiApp};

pub struct AppManager {
    apps: HashMap<&'static str, Box<dyn UiApp>>,
    active_id: &'static str,
}

impl AppManager {
    pub fn new(apps: HashMap<&'static str, Box<dyn UiApp>>, active_id: &'static str) -> Self {
        Self { apps, active_id }
    }

    pub fn render(&self, frame: &mut Frame) {
        if let Some(app) = self.apps.get(&self.active_id) {
            app.render(frame);
        }
    }

    pub fn update(&mut self, msg: &AppMsg) {
        for app in self.apps.values_mut() {
            app.update(msg);
        }
    }
}
