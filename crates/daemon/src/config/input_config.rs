use std::{borrow::Cow, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum InputConfig {
    #[default]
    Auto,
    Path(PathBuf),
    Name(Cow<'static, str>),
    OneOf(Vec<Cow<'static, str>>),
}
