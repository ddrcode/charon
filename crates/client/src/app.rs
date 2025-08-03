use std::sync::Arc;

use charon_lib::event::{DomainEvent, Event as DaemonEvent};
use eyre::OptionExt;
use ratatui::layout::Rect;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
    select,
    sync::mpsc,
    task::spawn_blocking,
    time::Instant,
};
use tracing::{debug, error, warn};

use crate::{
    domain::{AppEvent, Command, Context},
    root::AppManager,
    tui::{Event as TuiEvent, Tui},
};

pub struct App {
    ctx: Arc<Context>,
    app_mngr: AppManager,
    should_quit: bool,
    should_suspend: bool,
    app_event_tx: mpsc::UnboundedSender<AppEvent>,
    app_event_rx: mpsc::UnboundedReceiver<AppEvent>,
    command_tx: mpsc::UnboundedSender<Command>,
    command_rx: mpsc::UnboundedReceiver<Command>,
    sock_writer: Option<BufWriter<OwnedWriteHalf>>,
    prev_tick: Instant,
}

impl App {
    pub fn new(app_mngr: AppManager, ctx: Arc<Context>) -> eyre::Result<Self> {
        let (app_event_tx, app_event_rx) = mpsc::unbounded_channel();
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        Ok(Self {
            ctx,
            app_mngr,
            should_quit: false,
            should_suspend: false,
            app_event_tx,
            app_event_rx,
            command_tx,
            command_rx,
            sock_writer: None,
            prev_tick: Instant::now(),
        })
    }

    pub async fn run(&mut self) -> eyre::Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(2.0)
            .frame_rate(1.0);
        tui.enter()?;

        let sock = UnixStream::connect(&self.ctx.config.daemon_socket).await?;
        let (reader, writer) = sock.into_split();
        self.sock_writer = Some(BufWriter::new(writer));
        let mut reader = BufReader::new(reader);

        let command_tx = self.command_tx.clone();
        loop {
            self.tick(&mut tui, &mut reader).await?;
            if self.should_suspend {
                tui.suspend()?;
                command_tx.send(Command::ResumeApp)?;
                command_tx.send(Command::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    pub fn stop(&mut self) {
        self.should_quit = true;
    }

    pub async fn tick(
        &mut self,
        tui: &mut Tui,
        reader: &mut BufReader<OwnedReadHalf>,
    ) -> eyre::Result<()> {
        let mut line = String::new();
        select! {
            Some(event) = tui.next_event() => {
                self.handle_tui_event(event).await?;
            }
            Some(action) = self.app_event_rx.recv() => {
                self.handle_app_event(action, tui).await?;
            }
            Some(command) = self.command_rx.recv() => {
                self.handle_command(command, tui).await?;
            }
            Ok(bytes) = reader.read_line(&mut line) => {
                self.handle_daemon_event(bytes, line).await?;
            }
        }
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> eyre::Result<()> {
        tui.draw(|f| self.app_mngr.render(f))?;
        Ok(())
    }

    async fn handle_tui_event(&mut self, event: TuiEvent) -> eyre::Result<()> {
        let app_event_tx = self.app_event_tx.clone();
        let command_tx = self.command_tx.clone();
        match event {
            TuiEvent::Quit => app_event_tx.send(AppEvent::Quit)?,
            TuiEvent::Tick => self.handle_tick()?,
            TuiEvent::Render => command_tx.send(Command::Render)?,
            TuiEvent::Resize(x, y) => app_event_tx.send(AppEvent::Resize(x, y))?,
            TuiEvent::Key(key) => app_event_tx.send(AppEvent::Key(key))?,
            _ => {}
        }
        Ok(())
    }

    async fn handle_app_event(&mut self, event: AppEvent, tui: &mut Tui) -> eyre::Result<()> {
        if !matches!(
            event,
            AppEvent::Tick(..) | AppEvent::Backend(DomainEvent::CurrentStats(..))
        ) {
            debug!("{event:?}");
        }
        match event {
            AppEvent::Quit => self.should_quit = true,
            AppEvent::Resize(w, h) => self.handle_resize(tui, w, h)?,
            _ => {}
        }

        if let Some(cmd) = self.app_mngr.update(&event).await {
            self.command_tx.send(cmd)?;
        }
        Ok(())
    }

    async fn handle_command(&mut self, command: Command, tui: &mut Tui) -> eyre::Result<()> {
        match command {
            Command::SuspendTUI => tui.exit()?,
            Command::ResumeTUI => {
                tui.enter()?;
                tui.terminal.clear()?;
            }
            Command::SuspendApp => self.should_suspend = true,
            Command::ResumeApp => self.should_suspend = false,
            Command::ClearScreen => tui.terminal.clear()?,
            Command::Render => self.render(tui)?,
            Command::SendEvent(event) => self.send_to_daemon(&event).await?,
            Command::Quit => self.should_quit = true,
            Command::ExitApp => self.switch_app("menu")?,
            Command::RunApp(app) => self.switch_app(app)?,
            Command::RunExternal(path, args) => self.run_external(path, args, tui).await?,
        }
        Ok(())
    }

    async fn handle_daemon_event(&mut self, size: usize, line: String) -> eyre::Result<()> {
        if size == 0 {
            warn!("Connection closed by daemon");
            self.should_quit = true;
            return Ok(());
        }
        let event = serde_json::from_str::<DaemonEvent>(&line.trim())?;
        self.app_event_tx
            .send(AppEvent::Backend(event.payload.clone()))?;
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> eyre::Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn handle_tick(&mut self) -> eyre::Result<()> {
        let now = Instant::now();
        let delta = now - self.prev_tick;
        self.prev_tick = now;
        self.app_event_tx.send(AppEvent::Tick(delta))?;
        Ok(())
    }

    fn switch_app(&mut self, name: &'static str) -> eyre::Result<()> {
        if self.app_mngr.has_app(name) {
            self.app_mngr.set_active(name);
            self.app_event_tx.send(AppEvent::Activate)?;
        }
        Ok(())
    }

    async fn send_to_daemon(&mut self, payload: &DomainEvent) -> eyre::Result<()> {
        let writer = self
            .sock_writer
            .as_mut()
            .ok_or_eyre("sock_writer not initialized")?;
        let event = DaemonEvent::new("client".into(), payload.clone());
        let json = serde_json::to_string(&event)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        Ok(())
    }

    async fn run_external(
        &mut self,
        path: String,
        args: Vec<String>,
        tui: &mut Tui,
    ) -> eyre::Result<()> {
        tui.exit()?;
        let output = spawn_blocking(move || {
            std::process::Command::new(path)
                .args(args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
        })
        .await?;

        tui.enter()?;
        tui.terminal.clear()?;

        let output = output
            .inspect_err(|err| {
                error!("Error executing external app: {err}");
            })
            .ok();

        self.app_event_tx
            .send(AppEvent::ReturnFromExternal(output))?;

        Ok(())
    }
}
