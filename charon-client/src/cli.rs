use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(
        short,
        long,
        value_parser,
        default_value = default_config_path().into_os_string(),
        help = "Path to the configuration file"
    )]
    pub config: PathBuf,
}

fn default_config_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(std::env::var("XDG_CONFIG_HOME").unwrap_or("~/.config".into()));
    path.push("charon/tui.toml");
    path
}
