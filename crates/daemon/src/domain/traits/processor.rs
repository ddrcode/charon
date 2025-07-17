use charon_lib::event::Event;

#[async_trait::async_trait]
pub trait Processor: Send + Sync {
    async fn process(&mut self, event: Event) -> Vec<Event>;
}
