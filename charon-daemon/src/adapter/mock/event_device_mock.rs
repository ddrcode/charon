use evdev::{EventType, InputEvent, KeyCode};
use std::{collections::VecDeque, sync::Arc};
use tokio::{
    sync::Mutex,
    time::{Duration, sleep},
};

use crate::port::EventDevice;

#[derive(Default)]
pub struct EventDeviceState {
    pub grabbed: bool,
    pub grab_calls: u16,
    pub ungrab_calls: u16,
    pub events: VecDeque<InputEvent>,
}

impl EventDeviceState {
    pub fn simulate_key_press(&mut self, key_code: KeyCode) {
        let event = InputEvent::new_now(EventType::KEY.0, key_code.code(), 1);
        self.events.push_back(event);
    }

    pub fn simulate_key_release(&mut self, key_code: KeyCode) {
        let event = InputEvent::new_now(EventType::KEY.0, key_code.code(), 0);
        self.events.push_back(event);
    }

    /// Waits until all queued events have been consumed.
    pub async fn drain(state: &Arc<Mutex<Self>>) {
        while !state.lock().await.events.is_empty() {
            sleep(Duration::from_micros(100)).await;
        }
    }
}

#[derive(Default)]
pub struct EventDeviceMock {
    pub state: Arc<Mutex<EventDeviceState>>,
}

impl EventDeviceMock {
    pub fn state(&self) -> &Arc<Mutex<EventDeviceState>> {
        &self.state
    }
}

#[async_trait::async_trait]
impl EventDevice for EventDeviceMock {
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
