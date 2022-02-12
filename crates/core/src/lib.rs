#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;

pub mod aspell;
mod dictionary;
pub mod enchant;
pub mod ignore_store;
pub mod interactor;
pub mod os_io;
pub mod project;
pub mod repository;
pub mod sql;
pub mod tests;
pub mod tokens;

pub use crate::enchant::EnchantDictionary;
pub use aspell::AspellDictionary;
pub use checker::Checker;
pub use dictionary::Dictionary;
pub use ignore_store::IgnoreStore;
pub use interactor::{ConsoleInteractor, Interactor};
pub use os_io::{OperatingSystemIO, StandardIO};
pub use project::{Project, ProjectId, ProjectPath, RelativePath};
pub use repository::Repository;
pub use sql::{get_default_db_path, SQLRepository};
pub use tokens::TokenProcessor;
pub(crate) mod checker;
