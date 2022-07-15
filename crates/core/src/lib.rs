#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;

mod dictionary;
pub mod enchant;
pub mod ignore_file;
pub mod ignore_store;
pub mod kdl;
pub mod os_io;
pub mod project;
pub mod sql;
pub mod tests;
pub mod tokens;
pub mod undoer;

pub use crate::enchant::EnchantDictionary;
pub use checker::Checker;
pub use dictionary::Dictionary;
pub use ignore_file::IgnoreFile;
pub use ignore_store::{IgnoreStore, ProjectInfo};
pub use os_io::{OperatingSystemIO, StandardIO};
pub use project::{Project, ProjectId, ProjectPath, RelativePath};
pub use sql::{get_default_db_path, SQLRepository};
pub use tokens::TokenProcessor;
pub use undoer::{Operation, Undoer};
pub(crate) mod checker;
