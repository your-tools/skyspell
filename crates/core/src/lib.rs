#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;

pub mod config;
mod dictionary;
pub mod enchant;
pub mod ignore_file;
pub mod ignore_store;
pub mod os_io;
pub mod project;
pub mod repository;
pub mod sql;
pub mod storage;
pub mod tests;
pub mod tokens;
pub mod undo;

pub use crate::enchant::EnchantDictionary;
pub use checker::Checker;
pub use config::IgnoreConfig;
pub use dictionary::Dictionary;
pub use ignore_file::IgnoreFile;
pub use ignore_store::{IgnoreStore, ProjectInfo};
pub use os_io::{OperatingSystemIO, StandardIO};
pub use project::{Project, ProjectId, ProjectPath, RelativePath, SKYSPELL_IGNORE_FILE};
pub use repository::Repository;
pub use sql::{get_default_db_path, SQLRepository};
pub use storage::StorageBackend;
pub use tokens::TokenProcessor;
pub use undo::{Operation, Undoer};
pub(crate) mod checker;
