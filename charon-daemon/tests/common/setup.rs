use std::{
    collections::VecDeque,
    sync::{Arc, Mutex as StdMutex},
};

use charond::{
    actor::{KeyScanner, KeyWriter, Pipeline, Telemetry, Typist},
    adapter::{
        KeymapLoaderYaml,
        mock::{EventDeviceMock, HidDeviceMock, MetricsMock, MetricsState},
    },
    config::CharonConfig,
    domain::{ActorState, CharonEvent, Mode, Topic as CharonTopic, traits::Processor},
    port::KeymapLoader,
    processor::{KeyEventProcessor, SystemShortcutProcessor},
};
use maiko::{Actor, Subscribe, Supervisor, monitors::Tracer, testing::Harness};
use tracing_subscriber::FmtSubscriber;

use crate::common::MockKeyboard;

use super::TestContext;

pub struct MockedClient;
impl Actor for MockedClient {
    type Event = CharonEvent;
}

pub async fn setup() -> eyre::Result<TestContext> {
    use CharonTopic as T;
    init_logging();

    let config = CharonConfig::default();
    let keymap = KeymapLoaderYaml::new(&config.keymaps_dir)
        .load_keymap(&config.host_keymap)
        .await?;
    let state = ActorState::new(Mode::PassThrough, Arc::new(config));

    let mut sup = Supervisor::default();
    let test = Harness::new(&mut sup).await;

    let (scanner, keyboard_state) = {
        let input = EventDeviceMock::default();
        let keyboard = input.state().clone();
        let scanner = sup.add_actor(
            "KeyScanner",
            |ctx| KeyScanner::new(ctx, state.clone(), input, "test-keyboard".into()),
            Subscribe::none(),
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
        [T::KeyOutput],
    )?;

    let (telemetry, metrics) = {
        let state = Arc::new(StdMutex::new(MetricsState::default()));
        let actor = sup.add_actor(
            "Telemetry",
            |_ctx| Telemetry::new(MetricsMock::new(state.clone())),
            [T::Telemetry, T::KeyInput, T::Stats],
        )?;
        (actor, state)
    };

    let typist = sup.add_actor(
        "Typist",
        |ctx| Typist::new(ctx, state.clone(), keymap),
        [T::TextInput],
    )?;

    let client = sup.add_actor(
        "Client",
        |_ctx| MockedClient,
        [T::Client, T::Stats, T::Monitoring],
    )?;

    sup.monitors().add(Tracer).await;

    Ok(TestContext {
        sup,
        keyboard: MockKeyboard::new(keyboard_state),
        test,
        state,
        metrics,
        scanner,
        pipeline,
        writer,
        telemetry,
        typist,
        client,
    })
}

fn init_logging() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .with_target(false)
            .compact()
            .pretty()
            .without_time()
            .with_file(true)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Setting default subscriber failed");
    });
}
