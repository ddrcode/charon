use evdev::{EventType, InputEvent, KeyCode};
use std::{collections::VecDeque, sync::Arc};
use tokio::{
    sync::Mutex,
    time::{Duration, sleep},
};

use crate::port::AsyncInputSource;

#[derive(Default)]
pub struct MockState {
    pub grabbed: bool,
    pub grab_calls: u16,
    pub ungrab_calls: u16,
    pub events: VecDeque<InputEvent>,
}

impl MockState {
    pub fn simulate_key_press(&mut self, key_code: KeyCode) {
        let event = InputEvent::new_now(EventType::KEY.0, key_code.code(), 1);
        self.events.push_back(event);
    }

    pub fn simulate_key_release(&mut self, key_code: KeyCode) {
        let event = InputEvent::new_now(EventType::KEY.0, key_code.code(), 0);
        self.events.push_back(event);
    }
}

#[derive(Default)]
pub struct EvdevInputSourceMock {
    pub state: Arc<Mutex<MockState>>,
}

impl EvdevInputSourceMock {
    pub fn state(&self) -> &Arc<Mutex<MockState>> {
        &self.state
    }
}

#[async_trait::async_trait]
impl AsyncInputSource for EvdevInputSourceMock {
    async fn next_event(&mut self) -> Option<InputEvent> {
        sleep(Duration::from_millis(1)).await;
        let mut lock = self.state.lock().await;
        return lock.events.pop_front();
    }

    fn is_grabbed(&self) -> bool {
        let lock = self.state.try_lock().expect("Couldn't lock the state");
        return lock.grabbed;
    }

    fn grab(&mut self) -> std::io::Result<()> {
        let mut lock = self.state.try_lock().expect("Couldn't lock the state");
        lock.grabbed = true;
        lock.grab_calls += 1;
        Ok(())
    }

    fn ungrab(&mut self) -> std::io::Result<()> {
        let mut lock = self.state.try_lock().expect("Couldn't lock the state");
        lock.grabbed = false;
        lock.ungrab_calls += 1;
        Ok(())
    }
}
