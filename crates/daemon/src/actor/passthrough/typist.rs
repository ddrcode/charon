use tokio::fs::read_to_string;
use tracing::warn;

use crate::{
    devices::HIDKeyboard,
    domain::{HidKeyCode, KeyboardState},
};

pub struct Typist {
    state: KeyboardState,
    speed: tokio::time::Duration,
}

impl Typist {
    pub fn new(interval: u8) -> Self {
        Self {
            state: KeyboardState::new(),
            speed: tokio::time::Duration::from_millis(interval.into()),
        }
    }

    pub async fn send_char(&mut self, c: char, hidg: &mut HIDKeyboard) {
        let seq = match HidKeyCode::seq_from_char(c) {
            Ok(val) => val,
            Err(_) => {
                warn!("Couldn't produce sequence for char {c}");
                return;
            }
        };
        for key in seq.iter() {
            self.state.update_on_press(*key);
            hidg.send_report(&self.state.to_report());
            tokio::time::sleep(self.speed).await;
        }
        for key in seq.iter().rev() {
            self.state.update_on_release(*key);
            hidg.send_report(&self.state.to_report());
            tokio::time::sleep(self.speed).await;
        }
    }

    pub async fn send_string(&mut self, s: &String, hidg: &mut HIDKeyboard) {
        for c in s.chars() {
            self.send_char(c, hidg).await;
        }
    }

    pub async fn send_file(
        &mut self,
        path: &String,
        hidg: &mut HIDKeyboard,
    ) -> Result<(), std::io::Error> {
        let text = read_to_string(path).await?;
        self.send_string(&text, hidg).await;
        Ok(())
    }
}

impl Default for Typist {
    fn default() -> Self {
        Typist::new(50)
    }
}
