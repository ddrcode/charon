use std::{borrow::Cow, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum InputConfig {
    #[default]
    Auto,
    Path(PathBuf),
    Name(Cow<'static, str>),
    OneOf(Vec<Cow<'static, str>>),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CharonConfig {
    pub keyboard: InputConfig,
}

#[cfg(test)]
mod test {
    use super::*;
    use toml;

    #[test]
    fn serialize() {
        let config = CharonConfig {
            keyboard: InputConfig::OneOf(vec!["miso".into(), "pisio".into()]),
        };
        let s = toml::to_string(&config).unwrap();
        assert_eq!(s, "koza");
    }
}
