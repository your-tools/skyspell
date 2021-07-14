#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;

pub(crate) mod checker;
pub mod cli;
mod dictionary;
pub(crate) mod interactor;
pub mod kak;
pub(crate) mod models;
pub(crate) mod os_io;
pub(crate) mod project;
pub(crate) mod repository;
pub(crate) mod schema;
pub mod sql_repository;
pub(crate) mod token;

use checker::{Checker, InteractiveChecker, NonInteractiveChecker};
use dictionary::Dictionary;
pub use dictionary::EnchantDictionary;
use interactor::{ConsoleInteractor, Interactor};
pub use os_io::StandardIO;
use project::{Project, RelativePath};
use repository::Repository;
use token::TokenProcessor;

#[cfg(test)]
mod tests;
