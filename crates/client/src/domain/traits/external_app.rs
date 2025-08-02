use charon_lib::event::{DomainEvent, Mode};
use ratatui::Frame;

use crate::{
    components::notification,
    domain::{AppEvent, AppPhase, Command},
};

use super::UiApp;

#[async_trait::async_trait]
pub trait ExternalApp {
    fn id(&self) -> &'static str;
    async fn run(&mut self) -> Option<Command>;

    async fn on_start(&mut self) {}
    async fn on_finish(&mut self) {}
    async fn on_error(&mut self) {}

    fn phase(&self) -> AppPhase;
    fn set_phase(&mut self, state: AppPhase);
}

#[async_trait::async_trait]
impl<T> UiApp for T
where
    T: ExternalApp + Send + Sync,
{
    fn id(&self) -> &'static str {
        self.id()
    }

    async fn update(&mut self, msg: &AppEvent) -> Option<Command> {
        let cmd = match msg {
            AppEvent::Activate => {
                self.set_phase(AppPhase::Started);
                self.on_start().await;
                Command::SuspendTUI
            }
            AppEvent::TimerTick(_) => match self.phase() {
                AppPhase::Started => {
                    self.set_phase(AppPhase::Running);
                    let cmd = self.run().await;
                    if let Some(c) = &cmd
                        && matches!(c, Command::SendEvent(..))
                    {
                        self.set_phase(AppPhase::Closed);
                    } else {
                        self.set_phase(AppPhase::Closing);
                    }
                    return cmd;
                }
                AppPhase::Closed => {
                    self.set_phase(AppPhase::Sending);
                    Command::ResumeTUI
                }
                AppPhase::Closing => {
                    self.set_phase(AppPhase::Finishing);
                    Command::ResumeTUI
                }
                AppPhase::Finishing => {
                    self.on_finish().await;
                    self.set_phase(AppPhase::Done);
                    Command::RunApp("menu")
                }
                _ => return None,
            },
            AppEvent::Backend(DomainEvent::TextSent) => {
                self.on_finish().await;
                self.set_phase(AppPhase::Done);
                Command::SendEvent(DomainEvent::ModeChange(Mode::PassThrough))
            }
            _ => return None,
        };
        Some(cmd)
    }

    fn render(&self, f: &mut Frame) {
        if self.phase() == AppPhase::Sending {
            notification(
                f,
                "Please wait".into(),
                "Sending text...\nPress <[magic key]> to interrupt".into(),
            );
        }
    }
}
