use crate::schema::{extensions, files, good_words, ignored, ignored_for_ext, ignored_for_file};

#[derive(Insertable)]
#[table_name = "extensions"]
pub(crate) struct NewExtension<'a> {
    pub extension: &'a str,
}

#[derive(Insertable)]
#[table_name = "files"]
pub(crate) struct NewFile<'a> {
    pub full_path: &'a str,
}

#[derive(Insertable)]
#[table_name = "good_words"]
pub(crate) struct NewGoodWord<'a> {
    pub word: &'a str,
}

#[derive(Insertable)]
#[table_name = "ignored"]
pub(crate) struct NewIgnored<'a> {
    pub word: &'a str,
}

#[derive(Insertable)]
#[table_name = "ignored_for_ext"]
pub(crate) struct NewIgnoredForExt<'a> {
    pub word: &'a str,
    pub extension_id: i32,
}

#[derive(Insertable)]
#[table_name = "ignored_for_file"]
pub(crate) struct NewIgnoredForFile<'a> {
    pub word: &'a str,
    pub file_id: i32,
}

#[derive(Queryable)]
pub(crate) struct Ignored {
    pub id: i32,
    pub word: String,
}

#[derive(Queryable)]
pub(crate) struct IgnoredForExt {
    pub id: i32,
    pub word: String,
    pub extension_id: i32,
}

#[derive(Queryable)]
pub(crate) struct IgnoredForFile {
    pub id: i32,
    pub word: String,
    pub file_id: i32,
}

#[derive(Queryable)]
pub(crate) struct GoodWord {
    pub id: i32,
    pub word: String,
}

#[derive(Queryable)]
pub(crate) struct Extension {
    pub id: i32,
    pub extension: String,
}

#[derive(Queryable)]
pub(crate) struct File {
    pub id: i32,
    pub full_path: String,
}
