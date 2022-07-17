pub(crate) mod checker;
pub(crate) mod cli;
pub(crate) mod io;

pub use crate::checker::KakouneChecker;
pub use cli::main;
pub use io::{new_kakoune_io, KakouneIO, StdKakouneIO};
