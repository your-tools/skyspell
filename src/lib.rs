#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;

pub(crate) mod checker;
pub mod cli;
mod dictionary;
mod ignore;
pub(crate) mod interactor;
pub mod kak;
pub(crate) mod os_io;
pub(crate) mod project;
pub(crate) mod repository;
pub mod sql;
pub(crate) mod tokens;

use checker::{Checker, InteractiveChecker, NonInteractiveChecker};
use dictionary::Dictionary;
pub use dictionary::EnchantDictionary;
use ignore::Ignore;
use interactor::{ConsoleInteractor, Interactor};
pub use os_io::StandardIO;
use project::{Project, ProjectId, ProjectPath, RelativePath};
use repository::Repository;
use tokens::TokenProcessor;

#[cfg(test)]
mod tests;
