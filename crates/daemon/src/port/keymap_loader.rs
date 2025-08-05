use crate::{domain::Keymap, error::CharonError};

#[async_trait::async_trait]
pub trait KeymapLoader {
    async fn load_keymap(&self, name: &str) -> Result<Keymap, CharonError>;
}
