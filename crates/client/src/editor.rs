use std::process::Command;

use tempfile::NamedTempFile;

pub struct EditorSession {
    tmp_file: NamedTempFile,
}

impl EditorSession {
    pub fn new() -> Self {
        Self {
            tmp_file: NamedTempFile::new().expect("Couldn't create temp file"),
        }
    }

    pub fn run() -> anyhow::Result<()> {
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_owned();

        Command::new("nvim").arg(&path).status()?;

        Ok(())
    }

    fn send_file() {}
}
