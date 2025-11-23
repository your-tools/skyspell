#![deny(clippy::unwrap_used)]

mod checkers;
mod cli;
pub mod interactor;
#[cfg(test)]
mod tests;
pub use checkers::{InteractiveChecker, JsonChecker, NonInteractiveChecker};
pub use cli::{CheckOpts, main};
pub use interactor::{ConsoleInteractor, Interactor};

#[macro_export]
macro_rules! info_1 {
    ($($arg:tt)*) => ({
        use colored::*;
        println!("{} {}", "::".bold().blue(), format!($($arg)*));
    })
}

#[macro_export]
macro_rules! info_2 {
    ($($arg:tt)*) => ({
        use colored::*;

        println!("{} {}", "=>".bold().blue(), format!($($arg)*));
    })
}

#[macro_export]
macro_rules! info_3 {
    ($($arg:tt)*) => ({
        use colored::*;

        println!("{} {}", "*".bold().blue(), format!($($arg)*));
    })
}

#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => ({
        use colored::*;

        eprintln!("{} {}", "Error:".red(), format!($($arg)*));
    })
}
