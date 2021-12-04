table! {
    ignored (id) {
        id -> Integer,
        word -> Text,
    }
}

table! {
    ignored_for_extension (id) {
        id -> Integer,
        word -> Text,
        extension -> Text,
    }
}

table! {
    ignored_for_path (id) {
        id -> Integer,
        word -> Text,
        project_id -> Integer,
        path -> Text,
    }
}

table! {
    ignored_for_project (id) {
        id -> Integer,
        word -> Text,
        project_id -> Integer,
    }
}

table! {
    operations (id) {
        id -> Integer,
        json -> Text,
        date -> Integer,
    }
}

table! {
    projects (id) {
        id -> Integer,
        path -> Text,
    }
}

table! {
    skipped_file_names (id) {
        id -> Integer,
        file_name -> Text,
    }
}

table! {
    skipped_paths (id) {
        id -> Integer,
        project_id -> Integer,
        path -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    ignored,
    ignored_for_extension,
    ignored_for_path,
    ignored_for_project,
    operations,
    projects,
    skipped_file_names,
    skipped_paths,
);
