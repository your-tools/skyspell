table! {
    extensions (extension) {
        extension -> Text,
        programming_language_id -> Integer,
    }
}

table! {
    files (id) {
        id -> Integer,
        full_path -> Text,
    }
}

table! {
    ignored (id) {
        id -> Integer,
        word -> Text,
        file_id -> Nullable<Integer>,
        programming_language_id -> Nullable<Integer>,
    }
}

table! {
    programming_languages (id) {
        id -> Integer,
        name -> Text,
    }
}

joinable!(extensions -> programming_languages (programming_language_id));
joinable!(ignored -> files (file_id));

allow_tables_to_appear_in_same_query!(
    extensions,
    files,
    ignored,
    programming_languages,
);
