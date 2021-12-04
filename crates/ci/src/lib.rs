mod cli;
mod config;

const CONFIG_FILE_NAME: &str = "skyspell.yml";

pub use cli::main;
pub use config::parse_config;
