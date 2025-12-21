use charon_lib::event::DomainEvent;
use maiko::{Context, Meta};

use crate::domain::traits::Processor;

pub struct Pipeline {
    ctx: Context<DomainEvent>,
    processors: Vec<Box<dyn Processor + Send + Sync>>,
}

impl Pipeline {
    pub fn new(
        ctx: Context<DomainEvent>,
        processors: Vec<Box<dyn Processor + Send + Sync>>,
    ) -> Self {
        Self { ctx, processors }
    }

    async fn process(&mut self, event: &DomainEvent, meta: &Meta) -> maiko::Result<()> {
        let mut events = vec![event.clone()];

        for proc in &mut self.processors.iter_mut() {
            let mut next_events = Vec::new();
            for event in events {
                let mut out = proc.process(event, meta.clone()).await;
                next_events.append(&mut out);
            }
            events = next_events;
        }

        for ev in events {
            self.ctx.send(ev).await?;
        }

        Ok(())
    }
}

impl maiko::Actor for Pipeline {
    type Event = DomainEvent;

    async fn handle(&mut self, event: &Self::Event, meta: &maiko::Meta) -> maiko::Result<()> {
        if matches!(event, DomainEvent::Exit) {
            self.ctx.stop();
        } else {
            self.process(event, meta).await?;
        }
        Ok(())
    }
}
