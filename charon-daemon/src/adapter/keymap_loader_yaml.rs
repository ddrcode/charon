use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    str::FromStr as _,
};

use serde::Deserialize;
use tracing::debug;

use crate::{
    domain::{HidReport, KeyShortcut, Keymap},
    error::CharonError,
    port::KeymapLoader,
};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct OsVariantDto {
    pub os: HashSet<String>,
    pub mappings: HashMap<char, String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct KeymapDto {
    pub base: Option<String>,
    pub mappings: HashMap<char, String>,
    pub os_variants: Option<Vec<OsVariantDto>>,
}

pub struct KeymapLoaderYaml {
    keymaps_dir: String,
}

impl KeymapLoaderYaml {
    pub fn new(keymaps_dir: &str) -> Self {
        Self {
            keymaps_dir: String::from(keymaps_dir),
        }
    }

    async fn load_single_keymap(&self, name: &str) -> Result<Keymap, CharonError> {
        let mut path = PathBuf::new();
        path.push(&self.keymaps_dir);
        path.push(format!("{name}.yml"));
        debug!("Loading keymap from {path:?}");

        let data = tokio::fs::read_to_string(path).await?;
        let dto = serde_yaml_bw::from_str::<KeymapDto>(&data)?;
        let mappings = dto
            .mappings
            .iter()
            .map(|(key, val)| KeyShortcut::from_str(val).map(|s| (*key, HidReport::from(&s))))
            .collect::<Result<HashMap<_, _>, _>>()?;

        debug!("Keymap parsed succesfully");
        Ok(Keymap {
            name: String::from(name),
            base: dto.base,
            mappings,
        })
    }
}

#[async_trait::async_trait]
impl KeymapLoader for KeymapLoaderYaml {
    async fn load_keymap(&self, name: &str) -> Result<Keymap, CharonError> {
        let mut keymaps: Vec<Keymap> = Vec::new();
        let mut name = String::from(name);
        loop {
            let keymap = self.load_single_keymap(&name).await?;
            let maybe_parent = keymap.base.clone();
            keymaps.push(keymap);

            if let Some(parent) = maybe_parent {
                name = parent;
            } else {
                break;
            }
        }

        let mut result = keymaps[0].clone();
        result.mappings = keymaps
            .into_iter()
            .map(|km| km.mappings)
            .rev()
            .reduce(|mut acc, k| {
                acc.extend(k);
                acc
            })
            .expect("At least one keymap must be loaded");

        Ok(result)
    }
}
