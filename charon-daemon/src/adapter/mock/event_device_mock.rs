// SPDX-License-Identifier: GPL-3.0-or-later
use evdev::{EventType, InputEvent, KeyCode};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::{Mutex, Notify};

use crate::port::EventDevice;

pub struct EventDeviceState {
    pub grabbed: bool,
    pub grab_calls: u16,
    pub ungrab_calls: u16,
    pub events: VecDeque<InputEvent>,
    notify: Arc<Notify>,
}

impl EventDeviceState {
    fn new(notify: Arc<Notify>) -> Self {
        Self {
            grabbed: false,
            grab_calls: 0,
            ungrab_calls: 0,
            events: VecDeque::new(),
            notify,
        }
    }

    pub fn simulate_key_press(&mut self, key_code: KeyCode) {
        let event = InputEvent::new_now(EventType::KEY.0, key_code.code(), 1);
        self.events.push_back(event);
        self.notify.notify_one();
    }

    pub fn simulate_key_release(&mut self, key_code: KeyCode) {
        let event = InputEvent::new_now(EventType::KEY.0, key_code.code(), 0);
        self.events.push_back(event);
        self.notify.notify_one();
    }

    /// Waits until all queued events have been consumed.
    pub async fn drain(state: &Arc<Mutex<Self>>) {
        loop {
            let is_empty = state.lock().await.events.is_empty();
            if is_empty {
                return;
            }
            tokio::task::yield_now().await;
        }
    }
}

pub struct EventDeviceMock {
    pub state: Arc<Mutex<EventDeviceState>>,
    notify: Arc<Notify>,
}

impl Default for EventDeviceMock {
    fn default() -> Self {
        let notify = Arc::new(Notify::new());
        let state = Arc::new(Mutex::new(EventDeviceState::new(notify.clone())));
        Self { state, notify }
    }
}

impl EventDeviceMock {
    pub fn state(&self) -> &Arc<Mutex<EventDeviceState>> {
        &self.state
    }
}

impl EventDevice for EventDeviceMock {
    async fn next_event(&mut self) -> Option<InputEvent> {
        loop {
            {
                let mut lock = self.state.lock().await;
                if let Some(event) = lock.events.pop_front() {
                    return Some(event);
                }
            }
            // Block until notified - mirrors real epoll behavior
            self.notify.notified().await;
        }
    }

    fn is_grabbed(&self) -> bool {
        let lock = self.state.try_lock().expect("Couldn't lock the state");
        lock.grabbed
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
