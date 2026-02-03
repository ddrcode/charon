// SPDX-License-Identifier: GPL-3.0-or-later
/// Inspired by and partly copied from Ratatui's Component template,
/// although heavily modified and expanded.
/// Here is the [original version](https://github.com/ratatui/templates/blob/df2db86b0103e9ec66498f5523fa3fa40733b66b/component-generated/src/app.rs)
use std::{borrow::Cow, sync::Arc};

use charond::domain::CharonEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use eyre::OptionExt;
use maiko::{ActorId, Envelope};
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
use tracing::{debug, error, info, warn};

use crate::{
    components::notification,
    domain::{AppEvent, Command, Context, TickAction},
    root::AppManager,
    tui::{Event as TuiEvent, Tui},
};

pub struct App {
    ctx: Arc<Context>,
    app_mngr: AppManager,
    app_event_tx: mpsc::UnboundedSender<AppEvent>,
    app_event_rx: mpsc::UnboundedReceiver<AppEvent>,
    command_tx: mpsc::UnboundedSender<Command>,
    command_rx: mpsc::UnboundedReceiver<Command>,
    sock_writer: Option<BufWriter<OwnedWriteHalf>>,
    prev_tick: Instant,
    actor_id: ActorId,
}

impl App {
    pub fn new(app_mngr: AppManager, ctx: Arc<Context>) -> eyre::Result<Self> {
        let (app_event_tx, app_event_rx) = mpsc::unbounded_channel();
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        Ok(Self {
            ctx,
            app_mngr,
            app_event_tx,
            app_event_rx,
            command_tx,
            command_rx,
            sock_writer: None,
            prev_tick: Instant::now(),
            actor_id: ActorId::new("client".into()),
        })
    }

    pub async fn run(&mut self) -> eyre::Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(2.0)
            .frame_rate(1.0);
        tui.enter()?;

        let sock = self.connect_to_daemon(&mut tui).await?;
        let (reader, writer) = sock.into_split();
        self.sock_writer = Some(BufWriter::new(writer));
        let mut reader = BufReader::new(reader);
        let mut action;

        loop {
            action = self.tick(&mut tui, &mut reader).await?;
            match action {
                TickAction::Suspend => {
                    tui.suspend()?;
                    self.command_tx.send(Command::ResumeApp)?;
                    self.command_tx.send(Command::ClearScreen)?;
                    // tui.mouse(true);
                    tui.enter()?;
                }
                TickAction::Quit | TickAction::Upgrade => {
                    tui.stop()?;
                    break;
                }
                _ => {}
            }
        }
        tui.exit()?;
        if action == TickAction::Upgrade {
            info!(
                "Running upgrade script: {:?}",
                self.ctx.config.upgrade_script
            );
            use std::os::unix::process::CommandExt;
            let err = std::process::Command::new(&self.ctx.config.upgrade_script).exec();
            return Err(err.into()); // exec() only returns on error
        }
        Ok(())
    }

    pub async fn tick(
        &mut self,
        tui: &mut Tui,
        reader: &mut BufReader<OwnedReadHalf>,
    ) -> eyre::Result<TickAction> {
        let mut line = String::new();
        let action: TickAction;
        select! {
            event = tui.next_event() => {
                let event = event.ok_or_eyre("TUI event channel closed")?;
                action = self.handle_tui_event(event).await?;
            }
            event = self.app_event_rx.recv() => {
                let event = event.ok_or_eyre("App event channel closed")?;
                action = self.handle_app_event(event, tui).await?;
            }
            command = self.command_rx.recv() => {
                let command = command.ok_or_eyre("Command event channel closed")?;
                action = self.handle_command(command, tui).await?;
            }
            line_res = reader.read_line(&mut line) => {
                let bytes = line_res?;
                action = self.handle_daemon_event(bytes, line).await?;
            }
        }
        Ok(action)
    }

    fn render(&mut self, tui: &mut Tui) -> eyre::Result<()> {
        tui.draw(|f| self.app_mngr.render(f))?;
        Ok(())
    }

    async fn handle_tui_event(&mut self, event: TuiEvent) -> eyre::Result<TickAction> {
        let app_event_tx = self.app_event_tx.clone();
        let command_tx = self.command_tx.clone();
        match event {
            TuiEvent::Quit => app_event_tx.send(AppEvent::Quit)?,
            TuiEvent::Tick => self.handle_tick()?,
            TuiEvent::Render => command_tx.send(Command::Render)?,
            TuiEvent::Resize(x, y) => app_event_tx.send(AppEvent::Resize(x, y))?,
            TuiEvent::Key(key) => return self.handle_key_event(key),
            _ => {}
        }
        Ok(TickAction::None)
    }

    async fn handle_app_event(
        &mut self,
        event: AppEvent,
        tui: &mut Tui,
    ) -> eyre::Result<TickAction> {
        let mut action = TickAction::default();
        if !matches!(
            event,
            AppEvent::Tick(..) | AppEvent::Backend(CharonEvent::CurrentStats(..))
        ) {
            debug!("{event:?}");
        }
        match event {
            AppEvent::Quit => action = TickAction::Quit,
            AppEvent::Resize(w, h) => self.handle_resize(tui, w, h)?,
            _ => {}
        }

        if let Some(cmd) = self.app_mngr.update(&event).await {
            self.command_tx.send(cmd)?;
        }
        Ok(action)
    }

    async fn handle_command(
        &mut self,
        command: Command,
        tui: &mut Tui,
    ) -> eyre::Result<TickAction> {
        let mut action = TickAction::default();
        match command {
            Command::SuspendTUI => tui.exit()?,
            Command::ResumeTUI => {
                tui.enter()?;
                tui.terminal.clear()?;
            }
            Command::SuspendApp => action = TickAction::Suspend,
            Command::ResumeApp => {}
            Command::ClearScreen => tui.terminal.clear()?,
            Command::Render => self.render(tui)?,
            Command::SendEvent(event) => self.send_to_daemon(&event).await?,
            Command::Quit => action = TickAction::Quit,
            Command::ExitApp => self.switch_app("menu")?,
            Command::Upgrade => action = TickAction::Upgrade,
            Command::RunApp(app) => self.switch_app(app)?,
            Command::RunExternal(path, args) => self.run_external(path, args, tui).await?,
        }
        Ok(action)
    }

    async fn handle_daemon_event(&mut self, size: usize, line: String) -> eyre::Result<TickAction> {
        if size == 0 {
            warn!("Connection closed by daemon");
            return Ok(TickAction::Quit);
        }
        let msg_line = line.trim();
        if !msg_line.is_empty() {
            let envelope = serde_json::from_str::<Envelope<CharonEvent>>(msg_line)?;
            self.app_event_tx
                .send(AppEvent::Backend(envelope.event().clone()))?;
        }
        Ok(TickAction::None)
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

    fn handle_key_event(&mut self, key: KeyEvent) -> eyre::Result<TickAction> {
        let mut action = TickAction::default();
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                info!("Quitting on Ctrl+C");
                action = TickAction::Quit;
            }
            (KeyCode::Char('z'), KeyModifiers::CONTROL) => {
                info!("Suspending");
                action = TickAction::Suspend;
            }
            _ => self.app_event_tx.send(AppEvent::Key(key))?,
        }
        Ok(action)
    }

    fn switch_app(&mut self, name: &'static str) -> eyre::Result<()> {
        if self.app_mngr.has_app(name) {
            self.app_mngr.set_active(name);
            self.app_event_tx.send(AppEvent::Activate)?;
        }
        Ok(())
    }

    async fn send_to_daemon(&mut self, payload: &CharonEvent) -> eyre::Result<()> {
        let writer = self
            .sock_writer
            .as_mut()
            .ok_or_eyre("sock_writer not initialized")?;
        let event = Envelope::new(payload.clone(), self.actor_id.clone());
        let json = serde_json::to_string(&event)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        Ok(())
    }

    async fn run_external(
        &mut self,
        path: Cow<'static, str>,
        args: Vec<String>,
        tui: &mut Tui,
    ) -> eyre::Result<()> {
        tui.exit()?;
        let status = spawn_blocking(move || {
            std::process::Command::new(path.into_owned())
                .args(args)
                .status()
        })
        .await?;

        tui.enter()?;
        tui.terminal.clear()?;

        let status = status
            .inspect_err(|err| {
                error!("Error executing external app: {err}");
            })
            .ok();

        self.app_event_tx
            .send(AppEvent::ReturnFromExternal(status))?;

        Ok(())
    }

    async fn connect_to_daemon(&self, tui: &mut Tui) -> eyre::Result<UnixStream> {
        let socket_path = &self.ctx.config.daemon_socket;
        loop {
            match UnixStream::connect(socket_path).await {
                Ok(stream) => {
                    info!("Connected to daemon at {:?}", socket_path);
                    return Ok(stream);
                }
                Err(e) => {
                    info!("Waiting for daemon at {:?}... ({})", socket_path, e);
                    tui.draw(|f| {
                        notification(f, "Starting", "\nWaiting for Charon backend to start.\n")
                    })?;
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    }
}
