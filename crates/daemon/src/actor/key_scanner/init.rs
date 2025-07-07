use std::{fs, path::PathBuf};

use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tracing::info;

use super::KeyScanner;
use crate::domain::Event;

pub fn spawn_key_scanner(tx: Sender<Event>, rx: Receiver<Event>) -> JoinHandle<()> {
    let device_path = find_keyboard_device().unwrap();
    let mut scanner = KeyScanner::new(tx, rx, device_path);
    tokio::task::spawn_blocking(move || {
        scanner.run();
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
