use charon_lib::event::Mode;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    PassThrough,
    Menu,
    Popup(String, String),
}

pub struct App {
    pub name: String,
    pub icon: char,
    pub shortcut: String,
}

pub struct AppState {
    pub mode: Mode,
    pub screen: Screen,
    pub should_quit: bool,
    pub apps: Vec<App>,
    pub selected: usize,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            mode: Mode::PassThrough,
            screen: Screen::PassThrough,
            should_quit: false,
            apps: vec![
                App {
                    name: "Editor".into(),
                    icon: '\u{ed39}',
                    shortcut: "e".into(),
                },
                App {
                    name: "Stats".into(),
                    icon: '\u{f04c5}',
                    shortcut: "s".into(),
                },
                App {
                    name: "Passwords".into(),
                    icon: '\u{f07f5}',
                    shortcut: "p".into(),
                },
                App {
                    name: "Quit".into(),
                    icon: '\u{f0a48}',
                    shortcut: "q".into(),
                },
            ],
            selected: 0,
        }
    }

    pub fn switch_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
