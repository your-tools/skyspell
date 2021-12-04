#[macro_use]
extern crate lazy_static;

pub use checker::Checker;
pub use dictionary::Dictionary;
pub use interactor::{ConsoleInteractor, Interactor};
pub use os_io::{OperatingSystemIO, StandardIO};
pub use project::{Project, ProjectId, ProjectPath, RelativePath};
pub use repository::Repository;
pub use tokens::TokenProcessor;

pub(crate) mod checker;

mod dictionary;
pub mod interactor;
pub mod os_io;
pub mod project;
pub mod repository;
pub mod tokens;
