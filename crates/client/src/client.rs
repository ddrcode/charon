use charon_lib::event::{DomainEvent, Event, Mode};
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, layout::*, style::*, widgets::*};
use std::io::{self, Stdout};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixStream,
};

use crate::{
    app::{AppState, Screen},
    screen::{draw_menu, draw_pass_through, draw_popup},
};

pub struct CharonClient {
    state: AppState,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl CharonClient {
    pub fn new(state: AppState) -> Self {
        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
        Self { state, terminal }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        // Terminal setup
        let mut sock: UnixStream = UnixStream::connect("/tmp/charon.sock").await?;
        let (reader, mut writer) = sock.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        while !self.state.should_quit {
            self.redraw()?;
            tokio::select! {
                // Key input
                _ = tokio::task::spawn_blocking(|| event::poll(std::time::Duration::from_millis(50))) => {
                    if let CEvent::Key(key) = event::read()? {
                        self.handle_ui_input(key);
                    }
                }
                // Daemon stream
                Ok(bytes) = reader.read_line(&mut line) => {
                    if bytes == 0 {
                        self.state.quit(); // socket closed
                    } else {
                        let event: Event = serde_json::from_str(&line.trim()).unwrap();
                        self.handle_event(&event).await;
                    }
                    line.clear();
                }
            }
        }

        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn redraw(&mut self) -> io::Result<()> {
        self.terminal.draw(|f| match self.state.screen {
            Screen::PassThrough => draw_pass_through(f),
            Screen::Menu => draw_menu(f),
            Screen::Popup(ref msg) => draw_popup(f, msg),
        })?;
        Ok(())
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::Exit => self.state.quit(),
            DomainEvent::ModeChange(Mode::InApp) => self.state.switch_screen(Screen::Menu),
            DomainEvent::ModeChange(Mode::PassThrough) => {
                self.state.switch_screen(Screen::PassThrough)
            }
            DomainEvent::TextSent => self.state.switch_screen(Screen::Menu),
            _ => {}
        }
    }

    fn handle_ui_input(&mut self, key: KeyEvent) {
        match self.state.screen {
            Screen::Menu => {
                if key.code == KeyCode::Char('q') {
                    self.state.quit();
                }
                if key.code == KeyCode::Char('e') {
                    let text = self.run_editor().unwrap();
                    // self.state.switch_screen(Screen::Popup("Typing...".into()));
                }
            }
            _ => {}
        }
    }

    fn run_editor(&mut self) -> anyhow::Result<String> {
        use std::fs::read_to_string;
        use std::process::Command;
        use tempfile::NamedTempFile;

        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_owned();

        // execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        Command::new("nvim").arg(&path).status()?;

        let content = read_to_string(path)?;
        resume_tui(&mut self.terminal)?;
        self.terminal.clear()?;
        self.redraw()?;

        // let event = Event::new("client", DomainEvent::SendText(content));
        // broker_tx.send(event).await?;

        Ok(content)
        // Ok(path.to_string_lossy().into())
    }
}

pub fn resume_tui(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
