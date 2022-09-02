#[macro_use]
extern crate diesel;
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;

pub mod config;
mod dictionary;
pub mod enchant;
pub mod ignore_store;
pub mod operations;
pub mod os_io;
pub mod project;
pub mod repository;
pub mod skip_file;
pub mod sql;
pub mod storage;
pub mod tests;
pub mod tokens;

pub use crate::enchant::EnchantDictionary;
pub use checker::Checker;
pub use config::IgnoreConfig;
pub use dictionary::Dictionary;
pub use ignore_store::{IgnoreStore, ProjectInfo};
pub use operations::Operation;
pub use os_io::{OperatingSystemIO, StandardIO};
pub use project::{Project, ProjectId, ProjectPath, RelativePath, SKYSPELL_IGNORE_FILE};
pub use repository::Repository;
pub use skip_file::SkipFile;
pub use sql::{get_default_db_path, SQLRepository};
pub use storage::StorageBackend;
pub use tokens::TokenProcessor;
pub(crate) mod checker;
