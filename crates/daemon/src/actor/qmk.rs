use charon_lib::{event::CharonEvent, qmk::QMKEvent};
use maiko::{Context, Envelope, StepAction};
use tracing::debug;

use crate::{domain::ActorState, port::QmkDevice};

#[allow(dead_code)]
pub struct QMK {
    ctx: Context<CharonEvent>,
    state: ActorState,
    device: Box<dyn QmkDevice>,
}

#[allow(dead_code)]
impl QMK {
    pub fn new(ctx: Context<CharonEvent>, state: ActorState, device: Box<dyn QmkDevice>) -> Self {
        Self { ctx, state, device }
    }

    async fn process_qmk_event(&mut self, qmk_event: QMKEvent) -> maiko::Result {
        let event = match qmk_event {
            QMKEvent::ToggleMode => {
                let new_mode = self.state.mode().await.toggle();
                debug!("Switching mode to {:?}", new_mode);
                self.state.set_mode(new_mode).await;
                CharonEvent::ModeChange(new_mode)
            }
            QMKEvent::ModeChange(mode) => {
                self.state.set_mode(mode).await;
                CharonEvent::ModeChange(mode)
            }
            e => CharonEvent::QMKEvent(e),
        };
        self.ctx.send(event).await
    }
}

impl maiko::Actor for QMK {
    type Event = CharonEvent;

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> maiko::Result<()> {
        if matches!(envelope.event(), CharonEvent::Exit) {
            self.ctx.stop();
        }
        Ok(())
    }

    async fn step(&mut self) -> maiko::Result<StepAction> {
        while let Some(qmk_event) = self.device.read_event().await? {
            self.process_qmk_event(qmk_event).await?;
        }
        Ok(StepAction::Yield)
    }
}
