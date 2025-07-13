use std::{borrow::Cow, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum InputConfig {
    #[default]
    Auto,
    Path(PathBuf),
    Name(Cow<'static, str>),
    OneOf(Vec<Cow<'static, str>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharonConfig {
    pub keyboard: InputConfig,
    pub hid_keyboard: PathBuf,
    pub typing_interval: u8,
}

impl Default for CharonConfig {
    fn default() -> Self {
        Self {
            hid_keyboard: PathBuf::from("/dev/hidg0"),
            keyboard: InputConfig::default(),
            typing_interval: 20,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use toml;

    #[test]
    fn serialize() {
        let config = CharonConfig {
            keyboard: InputConfig::OneOf(vec!["keyb-1".into(), "keyb-2".into()]),
            ..Default::default()
        };
        let s = toml::to_string(&config);
        assert!(s.is_ok());
    }
}
