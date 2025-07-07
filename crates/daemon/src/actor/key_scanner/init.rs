use std::{fs, path::PathBuf};

use tokio::task::JoinHandle;
use tracing::info;

use super::KeyScanner;
use crate::domain::{Actor, ActorState};

pub fn spawn_key_scanner(state: ActorState) -> JoinHandle<()> {
    let device_path = find_keyboard_device().unwrap();
    let mut scanner = KeyScanner::new(state, device_path);
    tokio::task::spawn(async move {
        scanner.run().await;
    })
}

fn find_keyboard_device() -> Option<PathBuf> {
    let dir = "/dev/input/by-id";
    for entry in fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name();
        if name.to_string_lossy().ends_with("-event-kbd") {
            let full_path = fs::canonicalize(entry.path()).ok()?;
            info!("Keyboard found: {:?}", full_path);
            return Some(full_path);
        }
    }
    None
}
