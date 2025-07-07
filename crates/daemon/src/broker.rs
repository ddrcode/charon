use charon_lib::domain::{DomainEvent, Event};
use futures::{StreamExt, stream::FuturesUnordered};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::warn;

type Subscriber = (Sender<Event>, fn(&Event) -> bool);

pub struct EventBroker {
    alive: bool,
    receiver: Receiver<Event>,
    subscribers: Vec<Subscriber>,
}

impl EventBroker {
    pub fn new(receiver: Receiver<Event>) -> Self {
        EventBroker {
            alive: true,
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

    pub async fn run(&mut self) {
        while self.alive {
            match self.receiver.recv().await {
                Some(event) => {
                    let force = event.payload == DomainEvent::Exit;
                    self.broadcast(&event, force).await;
                    if force {
                        self.alive = false;
                    }
                }
                None => {
                    warn!("The global channel is no more.");
                    break;
                }
            }
        }
    }

    pub async fn broadcast(&self, event: &Event, force: bool) {
        let mut futures = FuturesUnordered::new();

        for (sender, filter) in &self.subscribers {
            if force || filter(event) {
                let evt = event.clone();
                let sender = sender.clone();
                futures.push(async move {
                    if let Err(e) = sender.send(evt).await {
                        warn!("Failed to send to subscriber: {}", e);
                    }
                });
            }
        }

        while let Some(_) = futures.next().await {}
    }
}
