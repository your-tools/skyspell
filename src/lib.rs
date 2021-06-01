#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

#[macro_use]
extern crate lazy_static;

pub(crate) mod checker;
pub mod cli;
pub(crate) mod db;
mod dictionary;
pub(crate) mod interactor;
pub mod kak;
pub(crate) mod models;
pub(crate) mod repo;
pub(crate) mod schema;
pub(crate) mod token;

use checker::{Checker, InteractiveChecker, NonInteractiveChecker};
use db::Db;
use dictionary::Dictionary;
use dictionary::EnchantDictionary;
use interactor::{ConsoleInteractor, Interactor};
use repo::Repo;
use token::{RelevantLines, Tokenizer};

#[cfg(test)]
mod tests;
