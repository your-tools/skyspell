use std::fmt::{Debug, Formatter};
use std::path::Path;

use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::models::*;
use crate::repo::Repo;
use crate::schema::extensions::dsl::{extension, extensions, id as extension_pk};
use crate::schema::files::dsl::{files, full_path, id as file_pk};
use crate::schema::ignored::dsl::{id as ignored_pk, ignored, word as ignored_word};
use crate::schema::ignored_for_ext::dsl::{
    extension_id as extension_fk, ignored_for_ext, word as ext_word,
};
use crate::schema::ignored_for_file::dsl::{
    file_id as file_fk, ignored_for_file, word as file_word,
};
use crate::schema::skipped_file_names::dsl::{
    file_name as skipped_file_name, id as skipped_file_name_id, skipped_file_names,
};
use crate::schema::skipped_paths::dsl::{
    full_path as skipped_path, id as skipped_path_id, skipped_paths,
};

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
    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()> {
        let new_ignored_words: Vec<_> = words.iter().map(|x| NewIgnored { word: x }).collect();
        diesel::insert_or_ignore_into(ignored)
            .values(new_ignored_words)
            .execute(&self.connection)?;
        Ok(())
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

    fn known_extension(&self, ext: &str) -> Result<bool> {
        Ok(extensions
            .filter(extension.eq(ext))
            .select(extension_pk)
            .first::<i32>(&self.connection)
            .optional()?
            .is_some())
    }

    fn known_file(&self, path: &str) -> Result<bool> {
        Ok(files
            .filter(full_path.eq(path))
            .select(file_pk)
            .first::<i32>(&self.connection)
            .optional()?
            .is_some())
    }

    fn add_ignored(&mut self, new_word: &str) -> Result<i32> {
        let new_word = &new_word.to_lowercase();
        let new_ignored = NewIgnored { word: new_word };
        diesel::insert_into(ignored)
            .values(new_ignored)
            .execute(&self.connection)?;
        Ok(ignored
            .filter(ignored_word.eq(new_word))
            .first::<Ignored>(&self.connection)?
            .id)
    }

    fn is_ignored(&self, word: &str) -> Result<bool> {
        Ok(ignored
            .filter(ignored_word.eq(word))
            .select(ignored_pk)
            .first::<i32>(&self.connection)
            .optional()?
            .is_some())
    }

    fn add_ignored_for_extension(&mut self, new_word: &str, existing_ext: &str) -> Result<()> {
        let new_word = &new_word.to_lowercase();
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
        let new_word = &new_word.to_lowercase();
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

    fn remove_ignored(&mut self, word: &str) -> Result<()> {
        diesel::delete(ignored)
            .filter(ignored_word.eq(word))
            .execute(&self.connection)?;
        Ok(())
    }

    fn remove_ignored_for_extension(&mut self, word: &str, ext: &str) -> Result<()> {
        let id = extensions
            .filter(extension.eq(ext))
            .select(extension_pk)
            .first::<i32>(&self.connection)
            .optional()?;

        let id = match id {
            None => return Ok(()),
            Some(i) => i,
        };

        diesel::delete(ignored_for_ext)
            .filter(extension_fk.eq(id))
            .filter(ext_word.eq(word))
            .execute(&self.connection)?;
        Ok(())
    }

    fn remove_ignored_for_file(&mut self, word: &str, file: &str) -> Result<()> {
        let id = files
            .filter(full_path.eq(file))
            .select(file_pk)
            .first::<i32>(&self.connection)
            .optional()?;

        let id = match id {
            None => return Ok(()),
            Some(i) => i,
        };

        diesel::delete(ignored_for_file)
            .filter(file_fk.eq(id))
            .filter(file_word.eq(word))
            .execute(&self.connection)?;
        Ok(())
    }

    fn skip_file_name(&mut self, new_file_name: &str) -> Result<()> {
        let new_skipped = NewSkippedFileName {
            file_name: new_file_name,
        };
        diesel::insert_into(skipped_file_names)
            .values(new_skipped)
            .execute(&self.connection)?;
        Ok(())
    }

    fn unskip_file_name(&mut self, file_name: &str) -> Result<()> {
        diesel::delete(skipped_file_names)
            .filter(skipped_file_name.eq(file_name))
            .execute(&self.connection)?;
        Ok(())
    }

    fn skip_full_path(&mut self, new_full_path: &str) -> Result<()> {
        let new_skipped = NewSkippedPath {
            full_path: new_full_path,
        };
        diesel::insert_into(skipped_paths)
            .values(new_skipped)
            .execute(&self.connection)?;
        Ok(())
    }

    fn unskip_full_path(&mut self, path: &str) -> Result<()> {
        diesel::delete(skipped_paths)
            .filter(skipped_path.eq(path))
            .execute(&self.connection)?;
        Ok(())
    }

    fn is_skipped(&self, path: &Path) -> Result<bool> {
        let full_path_ = match path.to_str() {
            None => return Ok(false),
            Some(f) => f,
        };

        // Look for the list of skipped paths
        if skipped_paths
            .filter(skipped_path.eq(full_path_))
            .select(skipped_path_id)
            .first::<i32>(&self.connection)
            .optional()?
            .is_some()
        {
            return Ok(true);
        }

        let file_name_ = match path.file_name().and_then(|x| x.to_str()) {
            None => return Ok(false),
            Some(n) => n,
        };

        // Look for the list of skipped file names
        if skipped_file_names
            .filter(skipped_file_name.eq(file_name_))
            .select(skipped_file_name_id)
            .first::<i32>(&self.connection)
            .optional()?
            .is_some()
        {
            return Ok(true);
        }

        Ok(false)
    }

    fn lookup_word(&self, query: &str, path: &Path) -> Result<bool> {
        let query = &query.to_lowercase();
        let full_path_ = path.to_str();
        let ext = path.extension().and_then(|x| x.to_str());

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
                    .filter(extension_fk.eq(know_ext.id))
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
                    .filter(file_fk.eq(known_file.id))
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
    fn test_db_lookup_in_ignored_words() {
        let mut db = Db::new(":memory:").unwrap();
        db.add_ignored("foobar").unwrap();

        assert!(db.lookup_word("foobar", &Path::new("-")).unwrap());
    }

    #[test]
    fn test_db_lookup_in_ignored_extensions() {
        let mut db = Db::new(":memory:").unwrap();
        db.add_ignored("foobar").unwrap();
        db.add_extension("py").unwrap();
        db.add_ignored_for_extension("defaultdict", "py").unwrap();

        assert!(db.lookup_word("defaultdict", &Path::new("foo.py")).unwrap());
    }

    #[test]
    fn test_db_lookup_in_files() {
        let mut db = Db::new(":memory:").unwrap();
        db.add_file("path/to/poetry.lock").unwrap();
        db.add_ignored_for_file("abcdef", "path/to/poetry.lock")
            .unwrap();

        assert!(db
            .lookup_word("abcdef", &Path::new("path/to/poetry.lock"))
            .unwrap());
    }

    #[test]
    fn test_db_lookup_in_skipped_file_names() {
        let mut db = Db::new(":memory:").unwrap();
        db.skip_file_name("poetry.lock").unwrap();

        assert!(db.is_skipped(&Path::new("path/to/poetry.lock")).unwrap());
    }

    #[test]
    fn test_db_remove_ignored() -> Result<()> {
        let mut db = Db::new(":memory:")?;
        db.add_ignored("foo")?;
        assert!(db.lookup_word("foo", Path::new("-'"))?);

        db.remove_ignored("foo")?;
        assert!(!db.lookup_word("foo", Path::new("-'"))?);
        Ok(())
    }

    #[test]
    fn test_db_remove_ignored_for_ext() -> Result<()> {
        let mut db = Db::new(":memory:")?;
        db.add_extension("py")?;
        db.add_extension("rs")?;
        db.add_ignored_for_extension("foo", "py")?;
        db.add_ignored_for_extension("foo", "rs")?;

        db.remove_ignored_for_extension("foo", "py")?;
        assert!(!db.lookup_word("foo", Path::new("foo.py"))?);
        assert!(db.lookup_word("foo", Path::new("foo.rs"))?);
        Ok(())
    }

    #[test]
    fn test_db_remove_ignored_for_file() -> Result<()> {
        let mut db = Db::new(":memory:")?;
        db.add_file("/path/to/one")?;
        db.add_file("/path/to/two")?;
        db.add_ignored_for_file("foo", "/path/to/one")?;
        db.add_ignored_for_file("foo", "/path/to/two")?;

        db.remove_ignored_for_file("foo", "/path/to/one")?;
        assert!(!db.lookup_word("foo", Path::new("/path/to/one"))?);
        assert!(db.lookup_word("foo", Path::new("/path/to/two"))?);
        Ok(())
    }
}
