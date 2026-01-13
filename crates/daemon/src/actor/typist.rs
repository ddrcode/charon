use charon_lib::event::{CharonEvent, Mode};
use deunicode::deunicode_char;
use maiko::{Context, Envelope};
use tokio::fs::{read_to_string, remove_file};
use tracing::{debug, warn};

use crate::{
    domain::{ActorState, HidReport, Keymap},
    error::CharonError,
};

pub struct Typist {
    ctx: Context<CharonEvent>,
    state: ActorState,
    speed: tokio::time::Duration,
    keymap: Keymap,
}

impl Typist {
    pub fn new(ctx: Context<CharonEvent>, state: ActorState, keymap: Keymap) -> Self {
        let interval = state.config().typing_interval;
        Self {
            ctx,
            state,
            speed: tokio::time::Duration::from_millis(interval.into()),
            keymap,
        }
    }

    fn to_ascii_report(&self, c: char) -> Option<&HidReport> {
        if let Some(decoded) = deunicode_char(c)
            && decoded.len() == 1
        {
            return self.keymap.report(
                decoded
                    .chars()
                    .next()
                    .expect("Expected string with only one character"),
            );
        }
        None
    }

    pub async fn send_char(&mut self, c: char) -> maiko::Result<()> {
        if let Some(report) = self.keymap.report(c).or_else(|| self.to_ascii_report(c)) {
            self.ctx.send(CharonEvent::HidReport(report.into())).await?;
            tokio::time::sleep(self.speed).await;
            self.ctx
                .send(CharonEvent::HidReport(HidReport::default().into()))
                .await?;
            tokio::time::sleep(self.speed).await;
        } else {
            warn!("Couldn't find key mapping for char {c}");
        }
        Ok(())
    }

    pub async fn send_string(&mut self, s: &String, source_id: &u128) -> maiko::Result<()> {
        for c in s.chars() {
            self.send_char(c).await?;
            if self.state.mode().await == Mode::PassThrough {
                warn!("Sending text interrupted by mode change");
                return Ok(());
            }
        }
        debug!("Typing completed");

        self.ctx
            .send_with_correlation(CharonEvent::TextSent, *source_id)
            .await
    }

    pub async fn send_file(
        &mut self,
        path: &String,
        remove: bool,
        source_id: &u128,
    ) -> Result<(), CharonError> {
        debug!("Typing text from file: {path}");
        let text = read_to_string(path).await?;
        self.send_string(&text, source_id).await?;
        if remove {
            remove_file(path).await?;
        }
        Ok(())
    }
}

impl maiko::Actor for Typist {
    type Event = CharonEvent;

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> maiko::Result<()> {
        let meta = envelope.meta();
        match envelope.event() {
            CharonEvent::SendText(txt) => self.send_string(txt, &meta.id()).await?,
            CharonEvent::SendFile(path, remove) => self
                .send_file(path, *remove, &meta.id())
                .await
                .expect("File not found"), // FIXME do we want to crash the system because of that?
            CharonEvent::Exit => self.ctx.stop(),
            _ => {}
        }
        Ok(())
    }
}
