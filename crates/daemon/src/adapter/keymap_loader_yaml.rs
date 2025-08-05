use crate::{domain::Keymap, error::CharonError, port::KeymapLoader};

pub struct KeymapLoaderYaml {}

#[async_trait::async_trait]
impl KeymapLoader for KeymapLoaderYaml {
    async fn load_keymap(&self, name: &str) -> Result<Keymap, CharonError> {
        let data = tokio::fs::read_to_string(name).await?;
        let keymap: Keymap = serde_yaml::from_str(&data)
            .map_err(|e| CharonError::YamlParseError(e, path.to_string()))?;
        Ok(keymap)
    }
}
