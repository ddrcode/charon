use charon_lib::domain::{DomainEvent, Event};

pub fn filter(event: &Event) -> bool {
    match event.payload {
        DomainEvent::ModeChange(_) => true,
        _ => false,
    }
}
