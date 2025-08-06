use std::{collections::HashMap, path::PathBuf, str::FromStr as _};

use serde::Deserialize;

use crate::{
    domain::{HidReport, KeyShortcut, Keymap},
    error::CharonError,
    port::KeymapLoader,
};

pub struct KeymapLoaderYaml {
    keymaps_dir: String,
}

impl KeymapLoaderYaml {
    pub fn new(keymaps_dir: &str) -> Self {
        Self {
            keymaps_dir: String::from(keymaps_dir),
        }
    }
}

#[derive(Deserialize)]
pub struct KeymapDto {
    pub name: String,
    pub base: Option<String>,
    pub mappings: HashMap<char, String>,
}

#[async_trait::async_trait]
impl KeymapLoader for KeymapLoaderYaml {
    async fn load_keymap(&self, name: &str) -> Result<Keymap, CharonError> {
        let mut path = PathBuf::new();
        path.push(&self.keymaps_dir);
        path.push(format!("{name}.yml"));
        let data = tokio::fs::read_to_string(name).await?;
        let dto = serde_yaml_bw::from_str::<KeymapDto>(&data)?;
        let mappings = dto
            .mappings
            .iter()
            .map(|(key, val)| KeyShortcut::from_str(val).map(|s| (*key, HidReport::from(&s))))
            .collect::<Result<HashMap<_, _>, _>>()?;
        Ok(Keymap {
            name: dto.name,
            base: dto.base,
            mappings,
        })
    }
}
