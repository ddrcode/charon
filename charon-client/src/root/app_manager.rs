use std::collections::HashMap;

use charond::domain::{CharonEvent, Mode};
use ratatui::Frame;
use tracing::{error, info};

use super::PassThroughController;
use crate::domain::{AppEvent, Command, traits::UiApp};

pub struct AppManager {
    apps: HashMap<&'static str, Box<dyn UiApp + Send + Sync>>,
    active_id: &'static str,
    mode: Mode,
    is_awake: bool,
    pass_through: PassThroughController,
}

impl AppManager {
    pub fn new(
        apps: HashMap<&'static str, Box<dyn UiApp + Send + Sync>>,
        active_id: &'static str,
    ) -> Self {
        Self {
            apps,
            active_id,
            mode: Mode::PassThrough,
            is_awake: true,
            pass_through: PassThroughController::default(),
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        if !self.is_awake {
            return;
        }
        if let Some(app) = self.apps.get(&self.active_id) {
            app.render(frame);
        }
    }

    pub async fn update(&mut self, msg: &AppEvent) -> Option<Command> {
        match msg {
            AppEvent::Backend(CharonEvent::ModeChange(mode)) => {
                self.mode = *mode;
                self.active_id = self.mode_screen();
                Some(Command::Render)
            }
            AppEvent::Backend(CharonEvent::Sleep) => {
                self.is_awake = false;
                None
            }
            AppEvent::Backend(CharonEvent::WakeUp) => {
                self.is_awake = true;
                None
            }
            m => {
                if !self.is_awake {
                    return None;
                }

                // In pass-through mode, let the controller handle layer events
                if self.mode == Mode::PassThrough {
                    if let Some(new_app) = self.pass_through.handle_event(m) {
                        if self.has_app(new_app) {
                            info!("PassThrough controller switching to: {new_app}");
                            self.active_id = new_app;
                            // Send Activate to the new app
                            if let Some(app) = self.apps.get_mut(&self.active_id) {
                                return app.update(&AppEvent::Activate).await;
                            }
                        }
                    }
                }

                // Forward event to active app
                if let Some(app) = self.apps.get_mut(&self.active_id) {
                    app.update(m).await
                } else {
                    None
                }
            }
        }
    }

    pub fn has_app(&self, app: &'static str) -> bool {
        self.apps.contains_key(app)
    }

    pub fn set_active(&mut self, app: &'static str) {
        if self.has_app(app) {
            info!("Activating app: {app}.");
            self.active_id = app;
        } else {
            error!("Couldn't find app: {app}");
        }
    }

    fn mode_screen(&self) -> &'static str {
        match self.mode {
            Mode::PassThrough => self.pass_through.active_app(),
            Mode::InApp => "menu",
        }
    }
}
