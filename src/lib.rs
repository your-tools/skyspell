#[macro_use]
extern crate lazy_static;

use checker::Checker;
use dictionary::Dictionary;
pub use dictionary::EnchantDictionary;
use interactor::{ConsoleInteractor, Interactor};
pub use os_io::StandardIO;
use project::{Project, ProjectId, ProjectPath, RelativePath};
use repository::Repository;
use tokens::TokenProcessor;

pub(crate) mod checker;

mod dictionary;
mod os_io;
pub(crate) mod project;
pub(crate) mod interactor;
pub(crate) mod repository;
pub(crate) mod tokens;
