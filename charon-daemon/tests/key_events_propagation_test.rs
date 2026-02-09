// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex as StdMutex},
};

use evdev::KeyCode;
use maiko::{ActorId, Envelope, Label, Supervisor, testing::Harness};
use tokio::sync::Mutex as TokioMutex;

use charond::{
    actor::{KeyScanner, KeyWriter, Pipeline, Telemetry},
    adapter::mock::{EventDeviceMock, EventDeviceState, HidDeviceMock, MetricsMock, MetricsState},
    config::CharonConfig,
    domain::{ActorState, CharonEvent, Mode, Topic as CharonTopic, traits::Processor},
    processor::{KeyEventProcessor, SystemShortcutProcessor},
};

struct MockKeyboard {
    state: Arc<TokioMutex<EventDeviceState>>,
}

impl MockKeyboard {
    fn new(state: Arc<TokioMutex<EventDeviceState>>) -> Self {
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
    pipeline: ActorId,
    writer: ActorId,
    telemetry: ActorId,
}

async fn setup() -> eyre::Result<TestContext> {
    use CharonTopic as T;
    let config = CharonConfig::default();
    let state = ActorState::new(Mode::PassThrough, Arc::new(config));

    let mut sup = Supervisor::default();
    let test = Harness::new(&mut sup).await;

    let (scanner, keyboard_state) = {
        let input = EventDeviceMock::default();
        let keyboard = input.state().clone();
        let scanner = sup.add_actor(
            "KeyScanner",
            |ctx| KeyScanner::new(ctx, state.clone(), input, "test-keyboard".into()),
            [T::System],
        )?;
        (scanner, keyboard)
    };

    let pipeline = sup.add_actor(
        "KeyEventPipeline",
        |ctx| {
            let processors: Vec<Box<dyn Processor + Send + Sync>> = vec![
                Box::new(KeyEventProcessor::default()),
                Box::new(SystemShortcutProcessor::new(ctx.clone(), state.clone())),
            ];
            Pipeline::new(ctx, processors)
        },
        [T::KeyInput],
    )?;

    let writer = sup.add_actor(
        "KeyWriter",
        |ctx| {
            let state = Arc::new(StdMutex::new(VecDeque::with_capacity(64)));
            let dev = HidDeviceMock::new(state);
            KeyWriter::new(ctx, dev)
        },
        [T::System, T::KeyOutput],
    )?;

    let telemetry = sup.add_actor(
        "Telemetry",
        |_ctx| {
            let state = Arc::new(StdMutex::new(MetricsState::default()));
            Telemetry::new(MetricsMock::new(state))
        },
        [T::System, T::Telemetry, T::KeyInput, T::Stats],
    )?;

    Ok(TestContext {
        sup,
        keyboard: MockKeyboard::new(keyboard_state),
        test,
        scanner,
        pipeline,
        writer,
        telemetry,
    })
}

#[tokio::test]
async fn test_key_press_emits_event() -> eyre::Result<()> {
    let mut ctx = setup().await?;

    println!("{}", ctx.sup.to_mermaid());

    ctx.sup.start().await?;

    ctx.test.start_recording().await;
    ctx.keyboard.key_press(KeyCode::KEY_S).await;
    ctx.keyboard.drain().await;
    ctx.test.stop_recording().await;

    let spy = ctx.test.actor(&ctx.scanner);
    assert_eq!(1, spy.outbound_count());
    let event = spy.last_sent().unwrap();
    let chain = ctx.test.chain(event.id());

    println!("\nActor chain for KeyPress S:");
    println!("{:?}", chain.actors().all());

    println!("\nMermaid diagram:");
    println!("{}", chain.to_mermaid());

    print!("\nEvent chain for KeyPress S: {}", chain.to_string_tree());

    assert!(chain.events().contains(event.id()));
    assert_eq!(chain.actors().path_count(), 2);
    assert!(chain.actors().path(&[&ctx.scanner, &ctx.telemetry]));
    assert!(
        chain
            .actors()
            .path(&[&ctx.scanner, &ctx.pipeline, &ctx.writer, &ctx.telemetry])
    );

    assert!(
        chain
            .events()
            .sequence(&["KeyPress", "HidReport", "ReportSent"])
    );

    ctx.sup.stop().await?;
    Ok(())
}
