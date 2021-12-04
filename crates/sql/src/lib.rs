#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub(crate) mod models;
mod repository;
pub mod schema;

pub use repository::{get_default_db_path, SQLRepository};

#[cfg(test)]
mod tests;
