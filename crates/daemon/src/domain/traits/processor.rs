use charon_lib::event::CharonEvent;
use maiko::Meta;

#[async_trait::async_trait]
pub trait Processor: Send + Sync {
    async fn process(&mut self, event: CharonEvent, meta: Meta) -> Vec<CharonEvent>;
}
