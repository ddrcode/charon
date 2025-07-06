use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

use super::PassThrough;
use crate::domain::{Actor, Event};

pub fn spawn_pass_through(tx: Sender<Event>, tr: Receiver<Event>) -> JoinHandle<()> {
    let mut passthrough = PassThrough::new(tx, tr);
    tokio::spawn(async move {
        passthrough.run().await;
    })
}
