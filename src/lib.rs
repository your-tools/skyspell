#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

#[macro_use]
extern crate lazy_static;

pub mod app;
pub(crate) mod checker;
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
use kak::KakouneChecker;
use repo::Repo;
use token::Tokenizer;

#[cfg(test)]
mod tests;
