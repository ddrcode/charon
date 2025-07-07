use tokio::task::JoinHandle;

use super::PassThrough;
use crate::domain::{Actor, ActorState};

pub fn spawn_pass_through(state: ActorState) -> JoinHandle<()> {
    let mut passthrough = PassThrough::new(state);
    tokio::spawn(async move {
        passthrough.run().await;
    })
}
