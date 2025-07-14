use std::{
    borrow::Cow,
    fs::{self, canonicalize, exists},
    path::PathBuf,
};

use tracing::{debug, info};

use crate::config::InputConfig;

const BY_ID: &str = "/dev/input/by-id/";

pub(crate) fn find_input_device(conf: &InputConfig) -> Option<PathBuf> {
    let maybe_device = match conf {
        InputConfig::Auto => find_keyboard_device(),
        InputConfig::Path(path) => normalize(path),
        InputConfig::Name(name) => from_name(name),
        InputConfig::OneOf(names) => names.iter().find_map(from_name),
    };

    if let Some(device) = &maybe_device {
        info!("Keyboard found: {:?}", device);
    } else {
        info!("Device not found for {:?}", conf);
    }

    maybe_device
}

fn normalize(path: &PathBuf) -> Option<PathBuf> {
    match exists(path) {
        Ok(true) => canonicalize(path).ok(),
        _ => None,
    }
}

fn from_name(name: &Cow<'static, str>) -> Option<PathBuf> {
    debug!("Searching for keyboard {}", name);
    let path: PathBuf = [BY_ID, name].iter().collect();
    normalize(&path)
}

pub fn find_keyboard_device() -> Option<PathBuf> {
    let dir = "/dev/input/by-id";
    for entry in fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name();
        if name.to_string_lossy().ends_with("-event-kbd") {
            let full_path = canonicalize(entry.path()).ok()?;
            return Some(full_path);
        }
    }
    None
}
