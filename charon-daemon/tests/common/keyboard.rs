use std::sync::Arc;

use charond::adapter::mock::EventDeviceState;
use evdev::KeyCode;
use tokio::sync::Mutex as TokioMutex;

pub struct MockKeyboard {
    state: Arc<TokioMutex<EventDeviceState>>,
}

impl MockKeyboard {
    pub fn new(state: Arc<TokioMutex<EventDeviceState>>) -> Self {
        Self { state }
    }

    pub async fn key_press(&self, key_code: KeyCode) {
        self.state.lock().await.simulate_key_press(key_code);
    }

    pub async fn key_release(&self, key_code: KeyCode) {
        self.state.lock().await.simulate_key_release(key_code);
    }

    pub async fn drain(&self) {
        EventDeviceState::drain(&self.state).await;
    }

    pub async fn is_grabbed(&self) -> bool {
        self.state.lock().await.grabbed
    }

    pub async fn grab_calls(&self) -> u16 {
        self.state.lock().await.grab_calls
    }

    pub async fn ungrab_calls(&self) -> u16 {
        self.state.lock().await.ungrab_calls
    }
}
