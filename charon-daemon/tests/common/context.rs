use std::sync::{Arc, Mutex};

use charond::{
    adapter::mock::MetricsState,
    domain::{ActorState, CharonEvent, Topic as CharonTopic},
};
use maiko::{ActorId, Supervisor, testing::Harness};

use super::MockKeyboard;

pub struct TestContext {
    pub sup: Supervisor<CharonEvent, CharonTopic>,
    pub test: Harness<CharonEvent, CharonTopic>,
    pub state: ActorState,
    pub keyboard: MockKeyboard,
    pub metrics: Arc<Mutex<MetricsState>>,

    pub scanner: ActorId,
    pub pipeline: ActorId,
    pub writer: ActorId,
    pub telemetry: ActorId,
    pub typist: ActorId,
    pub client: ActorId,
}
