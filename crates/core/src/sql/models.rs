use crate::sql::schema::*;

#[derive(Insertable)]
#[diesel(table_name = ignored)]
pub(crate) struct NewIgnored<'a> {
    pub word: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = ignored_for_extension)]
pub(crate) struct NewIgnoredForExtension<'a> {
    pub word: &'a str,
    pub extension: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = ignored_for_project)]
pub(crate) struct NewIgnoredForProject<'a> {
    pub word: &'a str,
    pub project_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = ignored_for_path)]
pub(crate) struct NewIgnoredForPath<'a> {
    pub word: &'a str,
    pub project_id: i32,
    pub path: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = projects)]
pub(crate) struct NewProject<'a> {
    pub path: &'a str,
}

#[derive(Queryable)]
pub struct ProjectModel {
    pub id: i32,
    pub path: String,
}

#[derive(Insertable)]
#[diesel(table_name = operations)]
pub(crate) struct NewOperation<'a> {
    pub json: &'a str,
    pub timestamp: i64,
}

#[derive(Queryable)]
pub struct OperationModel {
    pub id: i32,
    pub json: String,
    pub timestamp: i64,
}
