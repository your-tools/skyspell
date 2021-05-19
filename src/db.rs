use std::fmt::{Debug, Formatter};

use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::models::*;
use crate::repo::Repo;
use crate::schema::extensions::dsl::{extension, extensions};
use crate::schema::files::dsl::{files, full_path};
use crate::schema::good_words::dsl::{good_words, word as good_word};
use crate::schema::ignored::dsl::{ignored, word as ignored_word};
use crate::schema::ignored_for_ext::dsl::{extension_id, ignored_for_ext, word as ext_word};
use crate::schema::ignored_for_file::dsl::{file_id, ignored_for_file, word as file_word};

diesel_migrations::embed_migrations!("migrations");

pub fn new(url: &str) -> Result<Db> {
    Db::new(url)
}

pub struct Db {
    connection: SqliteConnection,
    url: String,
}

impl Debug for Db {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Db<{}>", self.url)
    }
}

impl Db {
    pub fn new(url: &str) -> Result<Self> {
        let connection = SqliteConnection::establish(&url)
            .with_context(|| format!("Could not connect to {}", url))?;
        embedded_migrations::run(&connection).with_context(|| "Could not migrate db")?;
        Ok(Self {
            connection,
            url: url.to_owned(),
        })
    }
}

impl Repo for Db {
    fn add_good_words(&mut self, words: &[&str]) -> Result<()> {
        let new_words: Vec<NewGoodWord> =
            words.into_iter().map(|x| NewGoodWord { word: x }).collect();
        diesel::insert_into(good_words)
            .values(new_words)
            .execute(&self.connection)?;
        Ok(())
    }

    fn add_ignored(&mut self, new_word: &str) -> Result<i32> {
        let new_ignored = NewIgnored { word: new_word };
        diesel::insert_into(ignored)
            .values(new_ignored)
            .execute(&self.connection)?;
        Ok(ignored
            .filter(ignored_word.eq(new_word))
            .first::<Ignored>(&self.connection)?
            .id)
    }

    fn add_extension(&mut self, new_ext: &str) -> Result<()> {
        let new_extension = NewExtension { extension: new_ext };
        diesel::insert_into(extensions)
            .values(new_extension)
            .execute(&self.connection)?;
        Ok(())
    }

    fn add_file(&mut self, new_file: &str) -> Result<()> {
        let new_file = NewFile {
            full_path: new_file,
        };
        diesel::insert_into(files)
            .values(new_file)
            .execute(&self.connection)?;
        Ok(())
    }

    fn add_ignored_for_extension(&mut self, new_word: &str, existing_ext: &str) -> Result<()> {
        let ext_in_db = extensions
            .filter(extension.eq(existing_ext))
            .first::<Extension>(&self.connection)?;

        let new_ignored_for_ext = NewIgnoredForExt {
            extension_id: ext_in_db.id,
            word: new_word,
        };

        diesel::insert_into(ignored_for_ext)
            .values(new_ignored_for_ext)
            .execute(&self.connection)?;

        Ok(())
    }

    fn add_ignored_for_file(&mut self, new_word: &str, existing_file: &str) -> Result<()> {
        let file_in_db = files
            .filter(full_path.eq(existing_file))
            .first::<File>(&self.connection)?;

        let new_ignored_for_file = NewIgnoredForFile {
            file_id: file_in_db.id,
            word: new_word,
        };

        diesel::insert_into(ignored_for_file)
            .values(new_ignored_for_file)
            .execute(&self.connection)?;

        Ok(())
    }

    fn lookup_word(&self, query: &str, file: Option<&str>, ext: Option<&str>) -> Result<bool> {
        let res = good_words
            .filter(good_word.eq(query))
            .first::<GoodWord>(&self.connection)
            .optional()?;

        // In the good_words table -> true
        if res.is_some() {
            return Ok(true);
        }

        // Is ignored globally -> true
        let res = ignored
            .filter(ignored_word.eq(query))
            .first::<Ignored>(&self.connection)
            .optional()?;

        if res.is_some() {
            return Ok(true);
        }

        // Look for the table specific to the ext (if given)
        if let Some(ext) = ext {
            let ext_in_db = extensions
                .filter(extension.eq(ext))
                .first::<Extension>(&self.connection)
                .optional()?;
            if let Some(know_ext) = ext_in_db {
                let res = ignored_for_ext
                    .filter(ext_word.eq(query))
                    .filter(extension_id.eq(know_ext.id))
                    .first::<IgnoredForExt>(&self.connection)
                    .optional()?;
                if res.is_some() {
                    return Ok(true);
                }
            }
        }

        // Look for the table specific to the file (if given)
        if let Some(file) = file {
            let file_in_db = files
                .filter(full_path.eq(file))
                .first::<File>(&self.connection)
                .optional()?;
            if let Some(known_file) = file_in_db {
                let res = ignored_for_file
                    .filter(file_word.eq(query))
                    .filter(file_id.eq(known_file.id))
                    .first::<IgnoredForFile>(&self.connection)
                    .optional()?;
                if res.is_some() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn known_extension(&self, ext: &str) -> Result<bool> {
        let ext_in_db = extensions
            .filter(extension.eq(ext))
            .first::<Extension>(&self.connection)
            .optional()?;

        Ok(ext_in_db.is_some())
    }

    fn known_file(&self, path: &str) -> Result<bool> {
        let file_in_db = files
            .filter(full_path.eq(path))
            .first::<File>(&self.connection)
            .optional()?;

        Ok(file_in_db.is_some())
    }

    fn has_good_words(&self) -> Result<bool> {
        let first = good_words.first::<GoodWord>(&self.connection).optional()?;
        Ok(first.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_lookup_in_good_words() {
        let mut db = Db::new(":memory:").unwrap();
        db.add_good_words(&["hello", "hi"]).unwrap();

        assert!(db.lookup_word("hello", None, None).unwrap());
    }

    #[test]
    fn test_db_lookup_in_ignored_words() {
        let mut db = Db::new(":memory:").unwrap();
        db.add_good_words(&["hello", "hi"]).unwrap();
        db.add_ignored("foobar").unwrap();

        assert!(db.lookup_word("foobar", None, None).unwrap());
    }

    #[test]
    fn test_db_lookup_in_ignored_extensions() {
        let mut db = Db::new(":memory:").unwrap();
        db.add_good_words(&["hello", "hi"]).unwrap();
        db.add_ignored("foobar").unwrap();
        db.add_extension("py").unwrap();
        db.add_ignored_for_extension("defaultdict", "py").unwrap();

        assert!(!db.lookup_word("defaultdict", None, None).unwrap());
    }

    #[test]
    fn test_db_lookup_in_files() {
        let mut db = Db::new(":memory:").unwrap();
        db.add_good_words(&["hello", "hi"]).unwrap();
        db.add_ignored("foobar").unwrap();
        db.add_extension("py").unwrap();
        db.add_file("poetry.lock").unwrap();
        db.add_ignored_for_file("abcdef", "poetry.lock").unwrap();

        assert!(db
            .lookup_word("abcdef", Some("poetry.lock"), Some("lock"))
            .unwrap());
    }
}
