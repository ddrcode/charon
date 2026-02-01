// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum InputConfig {
    #[default]
    Auto,
    Path(PathBuf),
    Name(String),
    OneOf(Vec<String>),
    Use(String),
}
