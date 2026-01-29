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

    async fn key_release(&self, key_code: KeyCode) {
        self.state.lock().await.simulate_key_release(key_code);
    }

    async fn drain(&self) {
        EventDeviceState::drain(&self.state).await;
    }

    async fn is_grabbed(&self) -> bool {
        self.state.lock().await.grabbed
    }

    async fn grab_calls(&self) -> u16 {
        self.state.lock().await.grab_calls
    }

    async fn ungrab_calls(&self) -> u16 {
        self.state.lock().await.ungrab_calls
    }
}

struct TestContext {
    sup: Supervisor<CharonEvent, CharonTopic>,
    test: Harness<CharonEvent, CharonTopic>,
    keyboard: MockKeyboard,
    sink: ActorId,
}

async fn setup() -> eyre::Result<TestContext> {
    setup_with_mode(Mode::PassThrough).await
}

async fn setup_with_mode(initial_mode: Mode) -> eyre::Result<TestContext> {
    use CharonTopic::*;
    let config = CharonConfig::default();
    let state = ActorState::new(initial_mode, Arc::new(config));

    let mut sup = Supervisor::default();
    let test = Harness::new(&mut sup).await;

    let keyboard_state = {
        let input = EventDeviceMock::default();
        let keyboard = input.state().clone();
        sup.add_actor(
            "KeyScanner",
            |ctx| KeyScanner::new(ctx, state.clone(), Box::new(input), "test-keyboard".into()),
            [System],
        )?;
        keyboard
    };

    let sink = sup.add_actor("Sink", |_ctx| Sink, [KeyInput])?;

    Ok(TestContext {
        sup,
        keyboard: MockKeyboard::new(keyboard_state),
        test,
        sink,
    })
}

impl TestContext {
    async fn switch_mode(&self, mode: Mode) -> maiko::Result<()> {
        self.test
            .send_as(&self.sink, CharonEvent::ModeChange(mode))
            .await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        Ok(())
    }
}

/// Tests that grab/ungrab is delayed until all keys are released.
/// This prevents stuck keys when switching modes mid-keystroke.
#[tokio::test]
async fn test_delayed_ungrab_after_key_release() -> eyre::Result<()> {
    let mut ctx = setup().await?;
    ctx.sup.start().await?;

    // Give actors time to initialize
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

    // Initial state - Charon is in PassThrough mode, so device should be grabbed
    assert!(
        ctx.keyboard.is_grabbed().await,
        "Device should be grabbed after init"
    );
    assert_eq!(1, ctx.keyboard.grab_calls().await);

    // Press a key (still in PassThrough mode)
    ctx.keyboard.key_press(KeyCode::KEY_A).await;
    ctx.keyboard.drain().await;

    // Switch to InApp mode while key is held
    // Device should remain grabbed until key is released
    ctx.switch_mode(Mode::InApp).await?;

    assert!(
        ctx.keyboard.is_grabbed().await,
        "Device should remain grabbed until all keys are released"
    );
    assert_eq!(0, ctx.keyboard.ungrab_calls().await);

    // Release the key - now ungrab should happen
    ctx.keyboard.key_release(KeyCode::KEY_A).await;
    ctx.keyboard.drain().await;

    assert!(
        !ctx.keyboard.is_grabbed().await,
        "Device should be ungrabbed after key release"
    );
    assert_eq!(1, ctx.keyboard.ungrab_calls().await);

    // Switching to same mode again shouldn't trigger another ungrab
    ctx.switch_mode(Mode::InApp).await?;
    assert!(!ctx.keyboard.is_grabbed().await);
    assert_eq!(1, ctx.keyboard.ungrab_calls().await);

    // Switch back to PassThrough - should grab immediately (no keys pressed)
    ctx.switch_mode(Mode::PassThrough).await?;

    assert!(ctx.keyboard.is_grabbed().await);
    assert_eq!(1, ctx.keyboard.ungrab_calls().await);
    assert_eq!(2, ctx.keyboard.grab_calls().await);

    ctx.sup.stop().await?;
    Ok(())
}

/// Tests that ALL keys must be released before ungrab happens.
/// Simulates: Ctrl+S shortcut where user releases S before Ctrl.
#[tokio::test]
async fn test_ungrab_waits_for_all_keys_released() -> eyre::Result<()> {
    let mut ctx = setup().await?;
    ctx.sup.start().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

    // Press two keys (simulating Ctrl+S)
    ctx.keyboard.key_press(KeyCode::KEY_LEFTCTRL).await;
    ctx.keyboard.drain().await;
    ctx.keyboard.key_press(KeyCode::KEY_S).await;
    ctx.keyboard.drain().await;

    // Switch to InApp mode
    ctx.switch_mode(Mode::InApp).await?;

    // Release S, but Ctrl still held - should stay grabbed
    ctx.keyboard.key_release(KeyCode::KEY_S).await;
    ctx.keyboard.drain().await;

    assert!(
        ctx.keyboard.is_grabbed().await,
        "Device should remain grabbed while Ctrl is still held"
    );
    assert_eq!(0, ctx.keyboard.ungrab_calls().await);

    // Release Ctrl - now ungrab should happen
    ctx.keyboard.key_release(KeyCode::KEY_LEFTCTRL).await;
    ctx.keyboard.drain().await;

    assert!(
        !ctx.keyboard.is_grabbed().await,
        "Device should be ungrabbed after all keys released"
    );
    assert_eq!(1, ctx.keyboard.ungrab_calls().await);

    ctx.sup.stop().await?;
    Ok(())
}

/// Tests that device is NOT grabbed when starting in InApp mode.
#[tokio::test]
async fn test_no_grab_when_starting_in_app_mode() -> eyre::Result<()> {
    let mut ctx = setup_with_mode(Mode::InApp).await?;
    ctx.sup.start().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

    assert!(
        !ctx.keyboard.is_grabbed().await,
        "Device should NOT be grabbed in InApp mode"
    );
    assert_eq!(0, ctx.keyboard.grab_calls().await);

    // Switch to PassThrough - should grab
    ctx.switch_mode(Mode::PassThrough).await?;

    assert!(ctx.keyboard.is_grabbed().await);
    assert_eq!(1, ctx.keyboard.grab_calls().await);

    ctx.sup.stop().await?;
    Ok(())
}

/// Tests that new key presses during pending ungrab are tracked.
/// Ungrab should wait for ALL keys including ones pressed after mode switch.
#[tokio::test]
async fn test_key_press_during_pending_ungrab() -> eyre::Result<()> {
    let mut ctx = setup().await?;
    ctx.sup.start().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

    // Press KEY_A
    ctx.keyboard.key_press(KeyCode::KEY_A).await;
    ctx.keyboard.drain().await;

    // Switch to InApp - ungrab is now pending
    ctx.switch_mode(Mode::InApp).await?;
    assert!(ctx.keyboard.is_grabbed().await);

    // Press another key while ungrab is pending
    ctx.keyboard.key_press(KeyCode::KEY_B).await;
    ctx.keyboard.drain().await;

    // Release KEY_A - KEY_B still held, should stay grabbed
    ctx.keyboard.key_release(KeyCode::KEY_A).await;
    ctx.keyboard.drain().await;

    assert!(
        ctx.keyboard.is_grabbed().await,
        "Device should remain grabbed while KEY_B is still held"
    );

    // Release KEY_B - now ungrab should happen
    ctx.keyboard.key_release(KeyCode::KEY_B).await;
    ctx.keyboard.drain().await;

    assert!(
        !ctx.keyboard.is_grabbed().await,
        "Device should be ungrabbed after all keys released"
    );
    assert_eq!(1, ctx.keyboard.ungrab_calls().await);

    ctx.sup.stop().await?;
    Ok(())
}
