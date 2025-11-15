pub(crate) mod checker;
pub(crate) mod cli;
pub(crate) mod io;

pub use crate::kak::checker::KakouneChecker;
pub use crate::kak::cli::main;
pub use crate::kak::io::{KakouneIO, new_kakoune_io};
