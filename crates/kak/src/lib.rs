pub(crate) mod checker;
pub mod cli;
pub(crate) mod io;

use crate::checker::KakouneChecker;
pub use io::{new_kakoune_io, KakouneIO, StdKakouneIO};
