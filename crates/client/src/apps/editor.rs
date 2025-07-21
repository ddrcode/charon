use std::sync::Arc;

use charon_lib::event::DomainEvent;
use ratatui::Frame;
use tokio::task::spawn_blocking;

use crate::{
    components::notification,
    domain::{AppMsg, Command, Context, traits::UiApp},
};

#[derive(Debug, PartialEq)]
enum EditorState {
    Started,
    EditorRunning,
    EditorClosed,
    Sending,
    Done,
    Error,
}

pub struct Editor {
    ctx: Arc<Context>,
    state: EditorState,
}

impl Editor {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self {
            ctx,
            state: EditorState::Started,
        }
    }

    pub fn new_box(ctx: Arc<Context>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Editor::new(ctx))
    }

    async fn run(&mut self) -> anyhow::Result<String> {
        self.state = EditorState::EditorRunning;
        use tempfile::NamedTempFile;

        let tmp = NamedTempFile::new()?;
        let path = tmp.into_temp_path().keep()?; // closes handle, keeps file alive
        let path_for_child = path.to_path_buf();

        spawn_blocking(move || {
            std::process::Command::new("nvim")
                .arg(&path_for_child)
                .status()
        })
        .await??;

        let path = path.to_string_lossy().to_string();
        Ok(path)
    }
}

#[async_trait::async_trait]
impl UiApp for Editor {
    fn id(&self) -> &'static str {
        "editor"
    }

    async fn update(&mut self, msg: &AppMsg) -> Option<Command> {
        let cmd = match msg {
            AppMsg::Activate => {
                self.state = EditorState::Started;
                Command::SuspendTUI
            }
            AppMsg::Deactivate => Command::ResumeTUI,
            AppMsg::TimerTick(_) if self.state != EditorState::Done => {
                match self.state {
                    EditorState::Started => {
                        let path = self.run().await.unwrap();
                        self.state = EditorState::EditorClosed;
                        return Some(Command::SendEvent(DomainEvent::SendFile(path, true)));
                    }
                    EditorState::EditorClosed => {
                        self.state = EditorState::Sending;
                        return Some(Command::ResumeTUI);
                    }
                    _ => {}
                }
                return None;
            }
            AppMsg::Backend(DomainEvent::TextSent) => {
                self.state = EditorState::Done;
                Command::RunApp("menu")
            }
            _ => return None,
        };
        Some(cmd)
    }

    fn render(&self, f: &mut Frame) {
        if self.state == EditorState::Sending {
            notification(
                f,
                "Please wait".into(),
                "Sending text...\nPress <[magic key]> to interrupt".into(),
            );
        }
    }
}
