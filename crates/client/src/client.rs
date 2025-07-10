use charon_lib::event::{DomainEvent, Event, Mode};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};
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
    screen::{draw_menu, draw_pass_through, draw_popup},
    tui::{resume_tui, suspend_tui},
};

pub struct CharonClient {
    state: AppState,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
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
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let mut line = String::new();

        self.redraw()?;

        // let (keyboard_tx, mut keyboard_rx) = tokio::sync::mpsc::channel(32);
        // let keyboard_loop_alive = Arc::new(AtomicBool::new(true));
        // {
        //     let keyboard_loop_alive = keyboard_loop_alive.clone();
        //     tokio::task::spawn_blocking(move || {
        //         while keyboard_loop_alive.load(Ordering::Relaxed) {
        //             if event::poll(std::time::Duration::from_millis(5)).unwrap() {
        //                 if let Ok(CEvent::Key(key)) = event::read() {
        //                     if let Err(_) = keyboard_tx.blocking_send(key) {
        //                         break;
        //                     }
        //                 }
        //             }
        //         }
        //     });
        // }

        while !self.state.should_quit {
            tokio::select! {
                // Some(key) = keyboard_rx.recv() => {
                //     self.handle_ui_input(key).await;
                //     self.redraw()?;
                // }
                Ok(bytes) = self.reader.read_line(&mut line) => {
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
        // keyboard_loop_alive.store(false, Ordering::Relaxed);

        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn redraw(&mut self) -> io::Result<()> {
        self.terminal.draw(|f| match self.state.screen {
            Screen::PassThrough => draw_pass_through(f),
            Screen::Menu => draw_menu(f, &self.state),
            Screen::Popup(ref title, ref msg) => draw_popup(f, title, msg),
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
            DomainEvent::KeyRelease(key) => self.handle_key_input(key).await,
            DomainEvent::TextSent => self.state.switch_screen(Screen::Menu),
            _ => {}
        }
    }

    // async fn handle_ui_input(&mut self, key: KeyEvent) {
    //     match self.state.screen {
    //         Screen::Menu => {
    //             if key.code == KeyCode::Char('q') {
    //                 self.state.quit();
    //             }
    //             if key.code == KeyCode::Char('e') {
    //                 let text = self.run_editor().await.unwrap();
    //             }
    //         }
    //         _ => {}
    //     }
    // }

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
        self.state.switch_screen(Screen::Popup(
            "Please wait".into(),
            "Sending text...\nPress <[magic key]> to interrupt".into(),
        ));

        let path = path.to_string_lossy().to_string();
        self.send(DomainEvent::SendFile(path, true)).await?;

        Ok(())
    }

    async fn send(&mut self, payload: DomainEvent) -> anyhow::Result<()> {
        let event = Event::new("client", payload);
        let json = serde_json::to_string(&event)?;
        self.writer.write_all(json.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }
}
