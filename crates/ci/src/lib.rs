use skyspell_core::ProjectId;
mod cli;
mod config;

const CONFIG_FILE_NAME: &str = "skyspell.yml";
// Note: we need a project_id (it's used in the Repository and IgnoreStore trait,
// but we only run with one project at a time ...
// TODO: find a better solution!
const PROJECT_ID: ProjectId = 42;

pub use cli::main;
pub use config::parse_config;
