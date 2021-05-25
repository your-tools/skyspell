#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

#[macro_use]
extern crate lazy_static;

pub mod checker;
pub mod db;
mod dictionary;
pub(crate) mod interactor;
pub(crate) mod models;
pub mod repo;
pub(crate) mod schema;
pub(crate) mod token;

pub use checker::{Checker, InteractiveChecker, NonInteractiveChecker};
pub use db::Db;
pub use dictionary::Dictionary;
pub use dictionary::EnchantDictionary;
pub use interactor::{ConsoleInteractor, Interactor};
pub use repo::Repo;
pub use token::Tokenizer;

#[cfg(test)]
mod tests;
