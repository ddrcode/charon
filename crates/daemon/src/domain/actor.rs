use tokio::sync::mpsc::Sender;

use crate::{
    domain::{DomainEvent, Event},
    error::KOSError,
};

#[async_trait::async_trait]
pub trait Actor {
    fn id() -> &'static str;

    fn sender(&self) -> &Sender<Event>;

    async fn send(&self, payload: DomainEvent) -> Result<(), KOSError> {
        let event = Event::new(Self::id(), payload);
        Ok(self.sender().send(event).await?)
    }

    async fn run(&mut self);
}
