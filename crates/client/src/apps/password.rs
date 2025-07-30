use std::sync::Arc;

use charon_lib::event::{DomainEvent, Mode};
use ratatui::Frame;
use tokio::{
    fs::{OpenOptions, read_to_string},
    io::AsyncWriteExt,
    task::spawn_blocking,
};
use tracing::error;

use crate::{
    components::notification,
    domain::{AppMsg, Command, Context, traits::UiApp},
};

#[derive(Debug, PartialEq)]
enum AppState {
    Started,
    Running,
    Closed,
    Sending,
    Done,
}

pub struct Password {
    ctx: Arc<Context>,
    state: AppState,
}

impl Password {
    pub fn new_box(ctx: Arc<Context>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Password {
            ctx,
            state: AppState::Started,
        })
    }

    async fn run(&mut self) -> anyhow::Result<String> {
        self.state = AppState::Running;
        spawn_blocking(|| std::process::Command::new("passepartui").status()).await??;
        let str = read_to_string(&self.ctx.config.clipboard_cache_file).await?;
        Ok(str)
    }

    async fn clear_cache(&self) -> std::io::Result<()> {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.ctx.config.clipboard_cache_file)
            .await?
            .write_all(b"")
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl UiApp for Password {
    fn id(&self) -> &'static str {
        "password"
    }

    async fn update(&mut self, msg: &AppMsg) -> Option<Command> {
        let cmd = match msg {
            AppMsg::Activate => {
                self.state = AppState::Started;
                Command::SuspendTUI
            }
            AppMsg::TimerTick(_) if self.state != AppState::Done => match self.state {
                AppState::Started => {
                    let cmd = match self.run().await {
                        Ok(pwd) => Command::SendEvent(DomainEvent::SendText(pwd)),
                        Err(err) => {
                            error!("Error getting password: {err}");
                            Command::RunApp("menu")
                        }
                    };
                    self.state = AppState::Closed;
                    cmd
                }
                AppState::Closed => {
                    self.state = AppState::Sending;
                    Command::ResumeTUI
                }
                _ => return None,
            },
            AppMsg::Backend(DomainEvent::TextSent) => {
                if let Err(e) = self.clear_cache().await {
                    error!("Failed clearing cache file: {e}");
                }
                self.state = AppState::Done;
                Command::SendEvent(DomainEvent::ModeChange(Mode::PassThrough))
            }
            _ => return None,
        };
        Some(cmd)
    }

    fn render(&self, f: &mut Frame) {
        if self.state == AppState::Sending {
            notification(
                f,
                "Please wait".into(),
                "Sending text...\nPress <[magic key]> to interrupt".into(),
            );
        }
    }
}
