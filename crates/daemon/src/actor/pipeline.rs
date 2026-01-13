use charon_lib::event::CharonEvent;
use maiko::{Context, Envelope, Meta};

use crate::domain::traits::Processor;

pub struct Pipeline {
    ctx: Context<CharonEvent>,
    processors: Vec<Box<dyn Processor + Send + Sync>>,
}

impl Pipeline {
    pub fn new(
        ctx: Context<CharonEvent>,
        processors: Vec<Box<dyn Processor + Send + Sync>>,
    ) -> Self {
        Self { ctx, processors }
    }

    async fn process(&mut self, event: &CharonEvent, meta: &Meta) -> maiko::Result<()> {
        let mut events = vec![event.clone()];

        for proc in &mut self.processors.iter_mut() {
            let mut next_events = Vec::new();
            for event in events {
                let mut out = proc.process(event, meta.clone()).await;
                next_events.append(&mut out);
            }
            events = next_events;
        }

        let correlation_id = meta.correlation_id().unwrap_or(meta.id());
        for event in events {
            self.ctx
                .send_envelope(Envelope::<CharonEvent>::with_correlation(
                    event,
                    self.ctx.clone_name(),
                    correlation_id,
                ))
                .await?;
        }

        Ok(())
    }
}

impl maiko::Actor for Pipeline {
    type Event = CharonEvent;

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> maiko::Result<()> {
        if matches!(envelope.event(), CharonEvent::Exit) {
            self.ctx.stop();
        } else {
            self.process(envelope.event(), envelope.meta()).await?;
        }
        Ok(())
    }
}
