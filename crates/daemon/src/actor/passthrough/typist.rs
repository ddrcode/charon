use std::fs::read_to_string;

pub struct Typist {
    text: String,
}

impl Typist {
    pub fn new(text: String) -> Self {
        Self { text }
    }

    pub fn from_file(path: String) -> Result<Self, std::io::Error> {
        Ok(Self {
            text: read_to_string(path)?,
        })
    }
}
