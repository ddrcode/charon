use std::sync::Arc;

use charond::domain::{CharonEvent, Mode, Topic as CharonTopic};
use evdev::KeyCode;
use maiko::{ActorId, Envelope, Supervisor, testing::Harness};
use tokio::sync::Mutex;

use charond::{
    actor::KeyScanner,
    adapter::mock::{EventDeviceMock, EventDeviceState},
    config::CharonConfig,
    domain::ActorState,
};

/// A no-op actor that subscribes to events for test observation.
struct Sink;

impl maiko::Actor for Sink {
    type Event = CharonEvent;
    async fn handle_event(&mut self, _: &Envelope<Self::Event>) -> maiko::Result<()> {
        Ok(())
    }
}

struct MockKeyboard {
    state: Arc<Mutex<EventDeviceState>>,
}

impl MockKeyboard {
    fn new(state: Arc<Mutex<EventDeviceState>>) -> Self {
        Self { state }
    }

    async fn key_press(&self, key_code: KeyCode) {
        self.state.lock().await.simulate_key_press(key_code);
    }

    async fn drain(&self) {
        EventDeviceState::drain(&self.state).await;
    }
}

struct TestContext {
    sup: Supervisor<CharonEvent, CharonTopic>,
    test: Harness<CharonEvent, CharonTopic>,
    keyboard: MockKeyboard,
    scanner: ActorId,
}

async fn setup() -> eyre::Result<TestContext> {
    use CharonTopic::*;
    let config = CharonConfig::default();
    let state = ActorState::new(Mode::PassThrough, Arc::new(config));

    let mut sup = Supervisor::default();
    let test = Harness::new(&mut sup).await;

    let (scanner, keyboard_state) = {
        let input = EventDeviceMock::default();
        let keyboard = input.state().clone();
        let scanner = sup.add_actor(
            "KeyScanner",
            |ctx| KeyScanner::new(ctx, state.clone(), Box::new(input), "test-keyboard".into()),
            [System],
        )?;
        (scanner, keyboard)
    };

    // Sink subscribes to KeyInput to observe events from KeyScanner
    sup.add_actor("Sink", |_ctx| Sink, [KeyInput])?;

    Ok(TestContext {
        sup,
        keyboard: MockKeyboard::new(keyboard_state),
        test,
        scanner,
    })
}

#[tokio::test]
async fn test_key_press_emits_event() -> eyre::Result<()> {
    let mut ctx = setup().await?;
    ctx.sup.start().await?;

    ctx.test.start_recording().await;
    ctx.keyboard.key_press(KeyCode::KEY_S).await;
    ctx.keyboard.drain().await;
    ctx.test.stop_recording().await;

    let spy = ctx.test.actor(&ctx.scanner);
    assert_eq!(1, spy.outbound_count());

    ctx.sup.stop().await?;
    Ok(())
}
