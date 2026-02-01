// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{domain::Keymap, error::CharonError};

pub trait KeymapLoader {
    fn load_keymap(&self, name: &str) -> impl Future<Output = Result<Keymap, CharonError>>;
}
