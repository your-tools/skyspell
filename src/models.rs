use crate::schema::{extensions, files, ignored, programming_languages};

#[derive(Insertable)]
#[table_name = "extensions"]
pub(crate) struct NewExtension<'a> {
    pub extension: &'a str,
    pub programming_language_id: i32,
}

#[derive(Insertable)]
#[table_name = "ignored"]
pub(crate) struct NewIgnored<'a> {
    pub word: &'a str,
    pub file_id: Option<i32>,
    pub programming_language_id: Option<i32>,
}

#[derive(Insertable)]
#[table_name = "programming_languages"]
pub(crate) struct NewProgrammingLanguage<'a> {
    pub name: &'a str,
}

#[derive(Insertable)]
#[table_name = "files"]
pub(crate) struct NewFile<'a> {
    pub full_path: &'a str,
}

#[derive(Queryable)]
pub(crate) struct File {
    pub id: i32,
    pub full_path: String,
}

#[derive(Queryable)]
pub(crate) struct Extension {
    pub extension: String,
    pub programming_language_id: i32,
}

#[derive(Queryable)]
pub(crate) struct Ignored {
    pub id: i32,
    pub word: String,
    pub file_id: Option<i32>,
    pub programming_language_id: Option<i32>,
}

#[derive(Queryable)]
pub(crate) struct ProgrammingLanguage {
    pub id: i32,
    pub name: String,
}
