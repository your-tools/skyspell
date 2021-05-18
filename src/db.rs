use std::fmt::{Debug, Formatter};

use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::models::*;
use crate::schema::extensions::dsl::{extension, extensions};
use crate::schema::files::dsl::*;
use crate::schema::ignored::dsl::*;
use crate::schema::programming_languages::dsl::*;

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

#[derive(Debug)]
pub enum AddFor {
    NaturaLanguage,
    ProgrammingLanguage(i32),
    File(i32),
}

#[derive(Debug)]
pub enum Query<'a> {
    Simple(&'a str),
    ForProgrammingLanguage(&'a str, i32),
    ForFile(&'a str, i32),
    ForFileOrProgrammingLanguage(&'a str, i32, i32),
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

    pub fn add_word(&self, new_word: &str, add_for: &AddFor) -> Result<()> {
        let (file, programming_language) = match &add_for {
            AddFor::NaturaLanguage => (None, None),
            AddFor::ProgrammingLanguage(p) => (None, Some(*p)),
            AddFor::File(f) => (Some(*f), None),
        };
        let new_ignored = NewIgnored {
            word: new_word,
            file_id: file,
            programming_language_id: programming_language,
        };
        diesel::insert_into(ignored)
            .values(new_ignored)
            .execute(&self.connection)?;
        Ok(())
    }

    pub fn add_programming_language(
        &self,
        new_language: &str,
        new_extensions: &[&str],
    ) -> Result<i32> {
        let new_programming_language = NewProgrammingLanguage { name: new_language };
        // Need to do it in two queries ...
        diesel::insert_into(programming_languages)
            .values(new_programming_language)
            .execute(&self.connection)?;

        let new_id = programming_languages
            .filter(name.eq(new_language))
            .first::<ProgrammingLanguage>(&self.connection)?
            .id;

        let new_extensions: Vec<_> = new_extensions
            .iter()
            .map(|x| NewExtension {
                extension: x,
                programming_language_id: new_id,
            })
            .collect();

        diesel::insert_into(extensions)
            .values(new_extensions)
            .execute(&self.connection)?;

        Ok(new_id)
    }

    pub fn add_file(&self, new_path: &str) -> Result<i32> {
        let new_file = NewFile {
            full_path: new_path,
        };
        diesel::insert_into(files)
            .values(new_file)
            .execute(&self.connection)?;

        let res = files
            .filter(full_path.eq(new_path))
            .first::<File>(&self.connection)?;
        Ok(res.id)
    }

    pub fn lookup_extension(&self, ext: &str) -> Result<Option<i32>> {
        let res = extensions
            .filter(extension.eq(ext))
            .first::<Extension>(&self.connection)
            .optional()?;
        Ok(res.map(|x| x.programming_language_id))
    }

    pub fn lookup_word(&self, query: &Query) -> Result<bool> {
        match query {
            Query::Simple(s) => self.search_word(s),
            Query::ForFile(s, f) => self.search_word_for_file(s, *f),
            Query::ForProgrammingLanguage(s, p) => self.search_word_for_programming_language(s, *p),
            Query::ForFileOrProgrammingLanguage(s, f, p) => {
                self.search_word_for_file_or_programming_language(s, *f, *p)
            }
        }
    }

    fn search_word(&self, query: &str) -> Result<bool> {
        let query = ignored
            .filter(word.eq(query))
            .filter(programming_language_id.is_null())
            .filter(file_id.is_null());

        let results = query.first::<Ignored>(&self.connection).optional()?;
        Ok(results.is_some())
    }

    fn search_word_for_file(&self, query: &str, file: i32) -> Result<bool> {
        let query = ignored.filter(word.eq(query)).filter(file_id.eq(file));

        let results = query.first::<Ignored>(&self.connection).optional()?;
        Ok(results.is_some())
    }

    fn search_word_for_programming_language(
        &self,
        query: &str,
        programming_language: i32,
    ) -> Result<bool> {
        let results = ignored
            .filter(word.eq(query))
            .filter(programming_language_id.eq(programming_language))
            .first::<Ignored>(&self.connection)
            .optional()?;
        Ok(results.is_some())
    }

    fn search_word_for_file_or_programming_language(
        &self,
        query: &str,
        programming_language: i32,
        file: i32,
    ) -> Result<bool> {
        let results = ignored
            .filter(word.eq(query))
            .filter(
                file_id
                    .eq(file)
                    .or(programming_language_id.eq(programming_language)),
            )
            .first::<Ignored>(&self.connection)
            .optional()?;
        Ok(results.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_programming_language() {
        let db = Db::new(":memory:").unwrap();
        let python = db.add_programming_language("Python", &["py"]).unwrap();
        let c_family = db
            .add_programming_language("C/C++", &["c", "cpp", "cc", "h", "hpp", "hxx"])
            .unwrap();

        assert_eq!(db.lookup_extension("c").unwrap().unwrap(), c_family);
        assert_eq!(db.lookup_extension("py").unwrap().unwrap(), python);
        assert!(db.lookup_extension("txt").unwrap().is_none())
    }

    #[test]
    fn test_lookup_ignored_word_for_natural_language() {
        let db = Db::new(":memory:").unwrap();
        db.add_word("foobar", &AddFor::NaturaLanguage).unwrap();

        let query = Query::Simple("foobar");
        assert!(db.lookup_word(&query).unwrap(), "foobar should be ignored");
    }

    #[test]
    fn test_lookup_ignored_word_for_programming_language() {
        let db = Db::new(":memory:").unwrap();

        let python = db.add_programming_language("Python", &["py"]).unwrap();

        db.add_word("foobar", &AddFor::NaturaLanguage).unwrap();
        db.add_word("defaultdict", &AddFor::ProgrammingLanguage(python))
            .unwrap();

        let query = Query::Simple("foobar");
        assert!(db.lookup_word(&query).unwrap(), "foobar is always ignored");

        let query = Query::Simple("defaultdict");
        assert!(
            !db.lookup_word(&query).unwrap(),
            "defaultdict  should not be ignored by default"
        );

        let query = Query::ForProgrammingLanguage("defaultdict", python);
        assert!(
            db.lookup_word(&query).unwrap(),
            "defaultdict  should be ignored for python files"
        );
    }

    #[test]
    fn test_lookup_ignored_word_for_file() {
        let db = Db::new(":memory:").unwrap();

        let lock = db.add_file("poetry.lock").unwrap();
        db.add_word("toto", &AddFor::File(lock)).unwrap();

        let query = Query::Simple("toto");
        assert!(
            !db.lookup_word(&query).unwrap(),
            "defaultdict should not be ignored by default"
        );

        let query = Query::ForFile("toto", lock);
        assert!(
            db.lookup_word(&query).unwrap(),
            "abcdef should be ignored for poetry.lock"
        );
    }

    #[test]
    fn test_lookup_ignored_word_for_file_or_programming_language() {
        let db = Db::new(":memory:").unwrap();

        let test_py = db.add_file("test.py").unwrap();
        let python = db.add_programming_language("python", &["py"]).unwrap();

        db.add_word("toto", &AddFor::File(test_py)).unwrap();
        db.add_word("defaultdict", &AddFor::ProgrammingLanguage(python))
            .unwrap();
        db.add_word("foobar", &AddFor::NaturaLanguage).unwrap();

        let query = Query::ForFileOrProgrammingLanguage("defaultdict", test_py, python);
        assert!(
            db.lookup_word(&query).unwrap(),
            "defaultdict should be ignored for test.py"
        );

        let query = Query::ForFileOrProgrammingLanguage("toto", test_py, python);
        assert!(
            db.lookup_word(&query).unwrap(),
            "toto should be ignored for test.py"
        );
    }
}
