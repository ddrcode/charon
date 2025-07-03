use crossbeam_channel::{Receiver, Sender};

use crate::domain::Event;

type Subscriber = (Sender<Event>, fn(&Event) -> bool);

pub struct EventBroker {
    receiver: Receiver<Event>,
    subscribers: Vec<Subscriber>,
}

impl EventBroker {
    pub fn new(receiver: Receiver<Event>) -> Self {
        EventBroker {
            receiver,
            subscribers: Vec::new(),
        }
    }

    pub fn add_subscriber(
        &mut self,
        sender: Sender<Event>,
        filter: fn(&Event) -> bool,
    ) -> &mut Self {
        self.subscribers.push((sender, filter));
        self
    }

    fn emit(&self, event: &Event) {
        for (sender, filter) in &self.subscribers {
            if filter(event) {
                sender.send(event.clone()).unwrap();
            }
        }
    }

    pub fn run(&self) {
        loop {
            match self.receiver.recv() {
                Ok(event) => {
                    self.emit(&event);
                }
                Err(e) => {
                    eprintln!("Error receiving event: {}", e);
                }
            }
        }
    }
}
