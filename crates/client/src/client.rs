use charon_lib::event::{DomainEvent, Event, Mode};
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, layout::*, style::*, widgets::*};
use std::{
    io::{self, Stdout, Write as _},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::{UnixStream, unix::WriteHalf},
};

use crate::{
    app::{AppState, Screen},
    screen::{draw_menu, draw_pass_through, draw_popup},
};

pub struct CharonClient {
    // sock: UnixStream,
    state: AppState,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    // writer: Option<WriteHalf<'a>>,
}

impl CharonClient {
    pub async fn new(state: AppState) -> Self {
        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
        Self {
            state,
            terminal,
            // writer: None,
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let mut sock = UnixStream::connect("/tmp/charon.sock").await.unwrap();
        let (reader, writer) = sock.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        self.redraw()?;

        let (keyboard_tx, mut keyboard_rx) = tokio::sync::mpsc::channel(32);
        let keyboard_loop_alive = Arc::new(AtomicBool::new(true));
        {
            let keyboard_loop_alive = keyboard_loop_alive.clone();
            tokio::task::spawn_blocking(move || {
                while keyboard_loop_alive.load(Ordering::Relaxed) {
                    if event::poll(std::time::Duration::from_millis(5)).unwrap() {
                        if let Ok(CEvent::Key(key)) = event::read() {
                            if let Err(_) = keyboard_tx.blocking_send(key) {
                                break;
                            }
                        }
                    }
                }
            });
        }

        while !self.state.should_quit {
            tokio::select! {
                Some(key) = keyboard_rx.recv() => {
                    self.handle_ui_input(key).await;
                    self.redraw()?;
                }
                Ok(bytes) = reader.read_line(&mut line) => {
                    if bytes == 0 {
                        self.state.quit(); // socket closed
                    } else {
                        let event: Event = serde_json::from_str(&line.trim()).unwrap();
                        self.handle_event(&event).await;
                        self.redraw()?;
                    }
                    line.clear();
                }
            }
        }
        keyboard_loop_alive.store(false, Ordering::Relaxed);

        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn redraw(&mut self) -> io::Result<()> {
        self.terminal.draw(|f| match self.state.screen {
            Screen::PassThrough => draw_pass_through(f),
            Screen::Menu => draw_menu(f, &self.state),
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

    async fn handle_ui_input(&mut self, key: KeyEvent) {
        match self.state.screen {
            Screen::Menu => {
                if key.code == KeyCode::Char('q') {
                    self.state.quit();
                }
                if key.code == KeyCode::Char('e') {
                    let text = self.run_editor().await.unwrap();
                    // self.state.switch_screen(Screen::Popup("Typing...".into()));
                }
            }
            _ => {}
        }
    }

    async fn run_editor(&mut self) -> anyhow::Result<()> {
        use std::process::Command;
        use tempfile::NamedTempFile;
        use tokio::fs::read_to_string;

        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_owned();

        // execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        Command::new("nvim").arg(&path).status()?;

        let content = read_to_string(path).await?;
        resume_tui(&mut self.terminal)?;
        self.terminal.clear()?;
        self.state
            .switch_screen(Screen::Popup("Sending text...".into()));
        self.redraw()?;

        // let event = Event::new("client", DomainEvent::SendText(content));
        // broker_tx.send(event).await?;
        self.send(DomainEvent::SendText(content)).await;
        Ok(())

        // Ok(path.to_string_lossy().into())
    }

    async fn send(&mut self, payload: DomainEvent) -> anyhow::Result<()> {
        // let event = Event::new("client", payload);
        // let json = serde_json::to_string(&event)?;
        // self.writer.unwrap().write_all(json.as_bytes()).await?;
        // self.writer.unwrap().write_all(b"\n").await?;
        Ok(())
    }
}

pub fn resume_tui(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
