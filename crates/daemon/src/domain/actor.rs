use crossbeam_channel::Sender;

use crate::{
    domain::{DomainEvent, Event},
    error::KOSError,
};

pub trait Actor {
    fn id() -> &'static str;

    fn sender(&self) -> &Sender<Event>;

    fn send(&self, payload: DomainEvent) -> Result<(), KOSError> {
        let event = Event::new(Self::id(), payload);
        Ok(self.sender().send(event)?)
    }

    fn run(&mut self);
}
