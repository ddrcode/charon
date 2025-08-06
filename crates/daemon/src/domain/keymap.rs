use std::collections::HashMap;

use crate::domain::HidReport;

pub struct Keymap {
    pub name: String,
    pub base: Option<String>,
    pub mappings: HashMap<char, HidReport>,
}

impl Keymap {
    pub fn new(name: String, base: Option<String>, mappings: HashMap<char, HidReport>) -> Self {
        Self {
            name,
            base,
            mappings,
        }
    }

    pub fn report(&self, c: char) -> Option<&HidReport> {
        self.mappings.get(&c)
    }
}
