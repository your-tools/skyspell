pub(crate) mod checker;
pub(crate) mod cli;
pub(crate) mod io;

use checker::KakouneChecker;
pub use io::{new_kakoune_io, StdKakouneIO};