use charon_lib::event::{DomainEvent, Event, Mode};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io::{self, Stdout},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
    task::spawn_blocking,
};

use crate::{
    app::{AppState, Screen},
    domain::PassThroughView,
    screen::{draw_menu, draw_pass_through, draw_popup},
    tui::{resume_tui, suspend_tui},
    util::DynamicInterval,
};

pub struct CharonClient {
    state: AppState,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
    next_refresh_timer: DynamicInterval,
}

impl CharonClient {
    pub fn new(state: AppState, stream: UnixStream) -> Self {
        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        let (reader, writer) = stream.into_split();
        let writer = BufWriter::new(writer);
        let reader = BufReader::new(reader);

        Self {
            state,
            terminal,
            reader,
            writer,
            next_refresh_timer: DynamicInterval::new(Duration::from_secs(60)),
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let mut line = String::new();
        let idle_interval = Duration::from_secs(30);

        self.redraw()?;

        while !self.state.should_quit {
            tokio::select! {
                Ok(bytes) = self.reader.read_line(&mut line) => {
                    if bytes == 0 {
                        self.state.quit(); // socket closed
                    } else {
                        let event: Event = serde_json::from_str(&line.trim()).unwrap();
                        self.handle_event(&event).await;
                    }
                    line.clear();
                }
                _ = tokio::time::sleep(idle_interval) => {
                    match self.state.screen {
                        Screen::PassThrough(_) => {
                            self.switch_screen(Screen::PassThrough(PassThroughView::Idle))?;
                        }
                        _ => {}
                    }
                }

                _ = self.next_refresh_timer.sleep_until() => {
                    if let Screen::PassThrough(PassThroughView::Splash) = self.state.screen {
                        self.state.switch_screen(Screen::PassThrough(PassThroughView::Charonsay));
                    }
                    self.redraw()?;
                    self.next_refresh_timer.reset();
                }
            }
        }

        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn redraw(&mut self) -> io::Result<()> {
        self.terminal.draw(|f| match self.state.screen {
            Screen::PassThrough(view) => draw_pass_through(f, view, &self.state.wisdoms),
            Screen::Menu => draw_menu(f, &self.state),
            Screen::Popup(ref title, ref msg) => draw_popup(f, title, msg),
        })?;
        Ok(())
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::Exit => self.state.quit(),
            DomainEvent::ModeChange(Mode::InApp) => {
                self.switch_screen(Screen::Menu).unwrap();
                self.next_refresh_timer.stop();
            }
            DomainEvent::ModeChange(Mode::PassThrough) => {
                self.switch_screen(Screen::PassThrough(PassThroughView::Splash))
                    .unwrap();
                self.next_refresh_timer.reset();
            }
            DomainEvent::KeyRelease(key, _) => self.handle_key_input(key).await,
            DomainEvent::TextSent => self.switch_screen(Screen::Menu).unwrap(),
            _ => {}
        }
    }

    async fn handle_key_input(&mut self, key: &evdev::KeyCode) {
        use evdev::KeyCode;
        match self.state.screen {
            Screen::Menu => {
                if *key == KeyCode::KEY_Q {
                    self.state.quit();
                }
                if *key == KeyCode::KEY_E {
                    self.run_editor().await.unwrap();
                }
            }
            Screen::PassThrough(PassThroughView::Idle) => {
                self.switch_screen(Screen::PassThrough(PassThroughView::Splash))
                    .unwrap();
                self.next_refresh_timer.reset();
            }
            _ => {}
        }
    }

    async fn run_editor(&mut self) -> anyhow::Result<()> {
        use std::process::Command;
        use tempfile::NamedTempFile;

        let tmp = NamedTempFile::new()?;
        let path = tmp.into_temp_path().keep()?; // closes handle, keeps file alive
        let path_for_child = path.to_path_buf();

        suspend_tui(&mut self.terminal)?;
        spawn_blocking(move || Command::new("nvim").arg(&path_for_child).status()).await??;
        resume_tui(&mut self.terminal)?;

        self.terminal.clear()?;
        self.redraw()?;
        self.switch_screen(Screen::Popup(
            "Please wait".into(),
            "Sending text...\nPress <[magic key]> to interrupt".into(),
        ))?;

        let path = path.to_string_lossy().to_string();
        self.send(DomainEvent::SendFile(path, true)).await?;

        Ok(())
    }

    async fn send(&mut self, payload: DomainEvent) -> anyhow::Result<()> {
        let event = Event::new("client".into(), payload);
        let json = serde_json::to_string(&event)?;
        self.writer.write_all(json.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }

    fn switch_screen(&mut self, screen: Screen) -> io::Result<()> {
        if screen != self.state.screen {
            self.state.switch_screen(screen);
            self.redraw()?;
        }
        Ok(())
    }
}
