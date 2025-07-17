use std::{borrow::Cow, sync::Arc};

use charon_lib::event::{DomainEvent, Event, Mode, Topic};
use tokio::{
    sync::{
        RwLock,
        mpsc::{self, Sender},
    },
    task::JoinHandle,
};
use tracing::{debug, info};

use crate::{
    actor::key_scanner::KeyScanner,
    broker::EventBroker,
    config::CharonConfig,
    domain::{Actor, ActorState},
};

pub struct Daemon {
    tasks: Vec<JoinHandle<()>>,
    broker: EventBroker,
    event_tx: Sender<Event>,
    mode: Arc<RwLock<Mode>>,
    config: CharonConfig,
}

impl Daemon {
    pub fn new() -> Self {
        let (event_tx, broker_rx) = mpsc::channel::<Event>(128);
        Self {
            tasks: Vec::new(),
            broker: EventBroker::new(broker_rx),
            event_tx,
            mode: Arc::new(RwLock::new(Mode::PassThrough)),
            config: CharonConfig::default(),
        }
    }

    pub async fn run(&mut self) {
        info!("Charon is ready...");
        self.broker.run().await;
        self.stop().await;
    }

    pub async fn stop(&mut self) {
        let event = Event::new("broker".into(), DomainEvent::Exit);
        self.broker.broadcast(&event, true).await;
    }

    pub async fn shutdown(&mut self) {
        for handle in self.tasks.drain(..) {
            handle.await.unwrap();
        }
    }

    fn register_actor<T: Actor>(
        &mut self,
        name: Cow<'static, str>,
        init: T::Init,
        topics: &'static [Topic],
        config: CharonConfig,
    ) -> &mut Self {
        let (pt_tx, pt_rx) = mpsc::channel::<Event>(128);
        self.broker.add_subscriber(pt_tx, name.clone(), topics);
        let state = ActorState::new(
            name,
            self.mode.clone(),
            self.event_tx.clone(),
            pt_rx,
            config,
        );
        let task = T::spawn(state, init);
        self.tasks.push(task);
        self
    }

    pub fn add_actor<T: Actor<Init = ()>>(&mut self, topics: &'static [Topic]) -> &mut Self {
        self.register_actor::<T>(T::name().into(), (), topics, self.config.clone())
    }

    pub fn add_actor_conditionally<T: Actor<Init = ()>>(
        &mut self,
        should_add: bool,
        topics: &'static [Topic],
    ) -> &mut Self {
        if should_add {
            self.add_actor::<T>(topics);
        }
        self
    }

    pub fn add_scanners(&mut self, topics: &'static [Topic]) -> &mut Self {
        for (name, config) in self.config.get_config_per_keyboard() {
            debug!("Registering scanner: {name}");
            self.register_actor::<KeyScanner>(
                format!("KeyScanner-{name}").into(),
                name,
                topics,
                config,
            );
        }
        self
    }

    pub fn add_actor_with_init<T: Actor>(
        &mut self,
        init: T::Init,
        topics: &'static [Topic],
    ) -> &mut Self {
        self.register_actor::<T>(T::name().into(), init, topics, self.config.clone())
    }

    pub fn update_config(&mut self, transform_cfg: fn(&mut CharonConfig)) -> &mut Self {
        (transform_cfg)(&mut self.config);
        self
    }

    pub fn with_config(&mut self, config: CharonConfig) -> &mut Self {
        self.config = config;
        self
    }
}
