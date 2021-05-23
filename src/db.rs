use std::fmt::{Debug, Formatter};
use std::path::Path;

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
use crate::schema::skipped_files::dsl::{file_name, skipped_files};

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
    fn insert_good_words(&mut self, words: &[&str]) -> Result<()> {
        let new_words: Vec<_> = words.iter().map(|x| NewGoodWord { word: x }).collect();
        diesel::insert_or_ignore_into(good_words)
            .values(new_words)
            .execute(&self.connection)?;
        Ok(())
    }

    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()> {
        let new_ignored_words: Vec<_> = words.iter().map(|x| NewIgnored { word: x }).collect();
        diesel::insert_or_ignore_into(ignored)
            .values(new_ignored_words)
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
        diesel::insert_or_ignore_into(extensions)
            .values(new_extension)
            .execute(&self.connection)?;
        Ok(())
    }

    fn add_file(&mut self, new_file: &str) -> Result<()> {
        let new_file = NewFile {
            full_path: new_file,
        };
        diesel::insert_or_ignore_into(files)
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

    fn skip_file_name(&mut self, new_file_name: &str) -> Result<()> {
        let new_skipped = NewSkippedFile {
            file_name: new_file_name,
        };
        diesel::insert_into(skipped_files)
            .values(new_skipped)
            .execute(&self.connection)?;
        Ok(())
    }

    fn lookup_word(&self, query: &str, path: &Path) -> Result<bool> {
        let full_path_ = path.to_str();
        let ext = path.extension().and_then(|x| x.to_str());
        let file_name_ = path.file_name().and_then(|f| f.to_str());

        // In the good_words table -> true
        let res = good_words
            .filter(good_word.eq(query))
            .first::<GoodWord>(&self.connection)
            .optional()?;
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

        // Look for the list of skipped file names
        if let Some(file_name_) = file_name_ {
            let filename_in_db = skipped_files
                .filter(file_name.eq(file_name_))
                .first::<SkippedFile>(&self.connection)
                .optional()?;
            if filename_in_db.is_some() {
                return Ok(true);
            }
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
        if let Some(f) = full_path_ {
            let file_in_db = files
                .filter(full_path.eq(f))
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_lookup_in_good_words() {
        let mut db = Db::new(":memory:").unwrap();
        db.insert_good_words(&["hello", "hi"]).unwrap();

        assert!(db.lookup_word("hello", &Path::new("-")).unwrap());
    }

    #[test]
    fn test_db_lookup_in_ignored_words() {
        let mut db = Db::new(":memory:").unwrap();
        db.insert_good_words(&["hello", "hi"]).unwrap();
        db.add_ignored("foobar").unwrap();

        assert!(db.lookup_word("foobar", &Path::new("-")).unwrap());
    }

    #[test]
    fn test_db_lookup_in_ignored_extensions() {
        let mut db = Db::new(":memory:").unwrap();
        db.insert_good_words(&["hello", "hi"]).unwrap();
        db.add_ignored("foobar").unwrap();
        db.add_extension("py").unwrap();
        db.add_ignored_for_extension("defaultdict", "py").unwrap();

        assert!(db.lookup_word("defaultdict", &Path::new("foo.py")).unwrap());
    }

    #[test]
    fn test_db_lookup_in_files() {
        let mut db = Db::new(":memory:").unwrap();
        db.insert_good_words(&["hello", "hi"]).unwrap();
        db.add_ignored("foobar").unwrap();
        db.add_extension("py").unwrap();
        db.add_file("poetry.lock").unwrap();
        db.add_ignored_for_file("abcdef", "poetry.lock").unwrap();

        assert!(db.lookup_word("abcdef", &Path::new("poetry.lock")).unwrap());
    }

    #[test]
    fn test_db_lookup_in_skipped() {
        let mut db = Db::new(":memory:").unwrap();
        db.insert_good_words(&["hello", "hi"]).unwrap();
        db.skip_file_name("poetry.lock").unwrap();

        assert!(db
            .lookup_word("abcdef", &Path::new("path/to/poetry.lock"))
            .unwrap());
    }
}
