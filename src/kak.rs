pub(crate) mod checker;
pub mod cli;
pub(crate) mod io;

use checker::KakouneChecker;
pub use io::{new_kakoune_io, KakouneIO, StdKakouneIO};
