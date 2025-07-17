use charon_lib::event::Event;

use crate::domain::ActorState;

#[async_trait::async_trait]
pub trait Processor: Send + Sync {
    async fn process(&mut self, input: Vec<Event>) -> Vec<Event>;
    fn set_state(&mut self, _state: &ActorState) {}
}
