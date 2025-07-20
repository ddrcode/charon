use charon_lib::event::{DomainEvent, Event};
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
use tracing::info;

use crate::{
    app::AppState,
    domain::{AppMsg, Command},
    root::AppManager,
    tui::{resume_tui, suspend_tui},
};

pub struct CharonClient {
    app_mngr: AppManager,
    state: AppState,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
}

impl CharonClient {
    pub fn new(app_mngr: AppManager, state: AppState, stream: UnixStream) -> Self {
        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        let (reader, writer) = stream.into_split();
        let writer = BufWriter::new(writer);
        let reader = BufReader::new(reader);

        Self {
            app_mngr,
            state,
            terminal,
            reader,
            writer,
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        info!("Client started");

        let mut line = String::new();
        let tick_duration = Duration::from_secs(1);
        let mut interval = tokio::time::interval(tick_duration);

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

                _ = interval.tick() => {
                    let cmd = self.app_mngr.update(&AppMsg::TimerTick(tick_duration)).await;
                    if let Some(cmd) = cmd {
                        self.handle_command(&cmd).await;
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        info!("Client quitting");
        Ok(())
    }

    fn redraw(&mut self) -> io::Result<()> {
        self.terminal.draw(|f| self.app_mngr.render(f))?;
        Ok(())
    }

    async fn handle_event(&mut self, event: &Event) {
        let cmd = self
            .app_mngr
            .update(&AppMsg::Backend(event.payload.clone()))
            .await;
        if let Some(cmd) = cmd {
            self.handle_command(&cmd).await;
        }
    }

    async fn handle_command(&mut self, command: &Command) {
        info!("Command to execute: {:?}", command);
        match command {
            Command::Render => self.redraw().unwrap(),
            Command::SendEvent(event) => self.send(event).await.unwrap(),
            Command::Exit => self.state.should_quit = true,
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
        // self.switch_screen(Screen::Popup(
        //     "Please wait".into(),
        //     "Sending text...\nPress <[magic key]> to interrupt".into(),
        // ))?;

        let path = path.to_string_lossy().to_string();
        self.send(&DomainEvent::SendFile(path, true)).await?;

        Ok(())
    }

    async fn send(&mut self, payload: &DomainEvent) -> anyhow::Result<()> {
        let event = Event::new("client".into(), payload.clone());
        let json = serde_json::to_string(&event)?;
        self.writer.write_all(json.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }
}
