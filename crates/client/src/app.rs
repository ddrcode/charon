use charon_lib::event::Mode;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    PassThrough,
    Menu,
    Popup(String),
}

pub struct AppState {
    pub mode: Mode,
    pub screen: Screen,
    pub should_quit: bool,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            mode: Mode::PassThrough,
            screen: Screen::PassThrough,
            should_quit: false,
        }
    }

    pub fn switch_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
