use charon_lib::event::DomainEvent;
use maiko::Meta;

#[async_trait::async_trait]
pub trait Processor: Send + Sync {
    async fn process(&mut self, event: DomainEvent, meta: Meta) -> Vec<DomainEvent>;
}
