#[macro_use]
extern crate lazy_static;

pub mod config;
mod dictionary;
pub mod enchant;
pub mod operations;
pub mod os_io;
pub mod project;
pub mod skip_file;
pub mod tests;
pub mod tokens;

pub use crate::enchant::EnchantDictionary;
pub use checker::Checker;
pub use config::IgnoreConfig;
pub use dictionary::Dictionary;
pub use operations::Operation;
pub use os_io::{OperatingSystemIO, StandardIO};
pub use project::{Project, ProjectPath, RelativePath, SKYSPELL_CONFIG_FILE};
pub use skip_file::SkipFile;
pub use tokens::TokenProcessor;
pub(crate) mod checker;
