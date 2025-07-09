use charon_lib::domain::{DomainEvent, Event, Topic};
use futures::{StreamExt, stream::FuturesUnordered};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{info, warn};

struct Subscriber {
    pub sender: Sender<Event>,
    pub name: &'static str,
    pub topics: &'static [Topic],
}

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
        name: &'static str,
        topics: &'static [Topic],
    ) -> &mut Self {
        self.subscribers.push(Subscriber {
            sender,
            name,
            topics,
        });
        self
    }

    pub async fn run(&mut self) {
        while self.alive {
            match self.receiver.recv().await {
                Some(event) => {
                    let force = event.payload == DomainEvent::Exit;
                    self.broadcast(&event, force).await;
                    if force {
                        self.stop().await;
                    }
                }
                None => {
                    warn!("The global channel is no more.");
                    break;
                }
            }
        }
        info!("EventBroker is stopping.");
    }

    pub async fn broadcast(&self, event: &Event, force: bool) {
        let mut futures = FuturesUnordered::new();
        let topic = event.payload.topic();

        for s in &self.subscribers {
            if !force && s.name == event.sender {
                continue;
            }
            if force || s.topics.contains(&topic) {
                let evt = event.clone();
                let sender = s.sender.clone();
                futures.push(async move {
                    if let Err(e) = sender.send(evt).await {
                        warn!("Failed to send to subscriber: {}", e);
                    }
                });
            }
        }

        while let Some(_) = futures.next().await {}
    }

    pub async fn stop(&mut self) {
        self.alive = false;
        self.receiver.close();
    }
}
