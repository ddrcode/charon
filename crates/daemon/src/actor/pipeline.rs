use charon_lib::event::DomainEvent;
use tokio::task::JoinHandle;

use crate::{
    domain::{ActorState, traits::Actor},
    error::CharonError,
};

pub struct Pipeline {
    state: ActorState,
}

#[async_trait::async_trait]
impl Actor for Pipeline {
    type Init = ();

    fn name() -> &'static str {
        "Pipeline"
    }

    fn spawn(state: ActorState, (): ()) -> Result<JoinHandle<()>, CharonError> {
        let mut actor = Pipeline { state };
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

            self.process_raw(event).await;
        }
    }

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }
}
