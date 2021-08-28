use crate::sql::schema::*;

#[derive(Insertable)]
#[table_name = "ignored"]
pub(crate) struct NewIgnored<'a> {
    pub word: &'a str,
}

#[derive(Insertable)]
#[table_name = "ignored_for_extension"]
pub(crate) struct NewIgnoredForExtension<'a> {
    pub word: &'a str,
    pub extension: &'a str,
}

#[derive(Insertable)]
#[table_name = "ignored_for_project"]
pub(crate) struct NewIgnoredForProject<'a> {
    pub word: &'a str,
    pub project_id: i32,
}

#[derive(Insertable)]
#[table_name = "ignored_for_path"]
pub(crate) struct NewIgnoredForPath<'a> {
    pub word: &'a str,
    pub project_id: i32,
    pub path: &'a str,
}

#[derive(Insertable)]
#[table_name = "skipped_file_names"]
pub(crate) struct NewSkippedFileName<'a> {
    pub file_name: &'a str,
}

#[derive(Insertable)]
#[table_name = "skipped_paths"]
pub(crate) struct NewSkippedPath<'a> {
    pub path: &'a str,
    pub project_id: i32,
}

#[derive(Insertable)]
#[table_name = "projects"]
pub(crate) struct NewProject<'a> {
    pub path: &'a str,
}

#[derive(Queryable)]
pub struct ProjectModel {
    pub id: i32,
    pub path: String,
}
