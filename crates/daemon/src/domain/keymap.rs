use std::collections::HashMap;

use crate::domain::HidReport;

pub struct Keymap {
    pub name: String,
    pub base: Option<String>,
    pub mappings: HashMap<char, HidReport>,
}
