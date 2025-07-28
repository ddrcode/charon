use charon_lib::event::DomainEvent;
use tokio::task::JoinHandle;

use crate::{
    domain::{
        ActorState,
        traits::{Actor, Processor},
    },
    error::CharonError,
};

pub struct Pipeline {
    state: ActorState,
    processors: Vec<Box<dyn Processor + Send + Sync>>,
}

#[async_trait::async_trait]
impl Actor for Pipeline {
    type Init = Vec<Box<dyn Processor + Send + Sync>>;

    fn name() -> &'static str {
        "Pipeline"
    }

    fn spawn(state: ActorState, processors: Self::Init) -> Result<JoinHandle<()>, CharonError> {
        let mut actor = Pipeline { state, processors };
        let handle = tokio::spawn(async move {
            actor.run().await;
        });
        Ok(handle)
    }

    async fn tick(&mut self) {
        if let Some(event) = self.state.receiver.recv().await {
            if event.payload == DomainEvent::Exit {
                self.stop().await;
                return;
            }

            let mut events = vec![event];

            for proc in &mut self.processors {
                let mut next_events = Vec::new();
                for event in events {
                    let mut out = proc.process(event).await;
                    next_events.append(&mut out);
                }
                events = next_events;
            }

            for mut ev in events {
                ev.sender = self.id();
                self.send_raw(ev).await;
            }
        }
    }

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }
}
