use std::{sync::Arc, time::Duration};

use charon_lib::event::{CharonEvent, Mode, Topic as CharonTopic};
use evdev::KeyCode;
use maiko::{ActorId, Supervisor, testing::Harness};
use tokio::{sync::Mutex, time::sleep};

use charond::{
    actor::KeyScanner,
    adapter::mock::{EventDeviceMock, EventDeviceState},
    config::CharonConfig,
    domain::ActorState,
};

struct MockKeyboard {
    state: Arc<Mutex<EventDeviceState>>,
}

impl MockKeyboard {
    fn new(state: Arc<Mutex<EventDeviceState>>) -> Self {
        Self { state }
    }

    async fn key_press(&self, key_code: KeyCode) {
        let mut state = self.state.lock().await;
        state.simulate_key_press(key_code);
    }

    async fn key_release(&self, key_code: KeyCode) {
        let mut state = self.state.lock().await;
        state.simulate_key_release(key_code);
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
            |ctx| {
                // let async_dev = AsyncFd::new(device).unwrap();
                KeyScanner::new(ctx, state.clone(), Box::new(input), "test-keyboard".into())
            },
            [System],
        )?;
        (scanner, keyboard)
    };

    Ok(TestContext {
        sup,
        keyboard: MockKeyboard::new(keyboard_state),
        test,
        scanner,
    })
}

#[tokio::test]
async fn test_key_send() -> eyre::Result<()> {
    let mut ctx = setup().await?;
    ctx.sup.start().await?;

    ctx.test.start_recording().await;
    ctx.keyboard.key_press(KeyCode::KEY_S).await;
    ctx.test.stop_recording().await;

    let spy = ctx.test.actor(&ctx.scanner);
    assert_eq!(1, spy.outbound_count());

    ctx.sup.stop().await?;
    Ok(())
}
