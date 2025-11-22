#[macro_use]
extern crate lazy_static;

mod dictionary;

#[cfg(target_family = "unix")]
#[path = "system_dictionary/unix.rs"]
mod system_dictionary;

#[cfg(target_family = "windows")]
#[path = "system_dictionary/windows.rs"]
mod system_dictionary;

pub use system_dictionary::SystemDictionary;

pub mod ignore;
pub mod operations;
pub mod os_io;
pub mod project;
pub mod skip_file;
pub mod tests;
pub mod tokens;

pub use checker::{Checker, CheckerState, ProcessOutcome, SpellingError};
pub use dictionary::Dictionary;
pub use ignore::{GlobalIgnore, IgnoreStore, LocalIgnore, global_path};
pub use operations::Operation;
pub use os_io::{OperatingSystemIO, StandardIO};
pub use project::{Project, ProjectFile, SKYSPELL_LOCAL_IGNORE};
pub use skip_file::SkipFile;
pub use tokens::{Position, Token, TokenProcessor};
pub(crate) mod checker;
