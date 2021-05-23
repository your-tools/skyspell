table! {
    extensions (id) {
        id -> Integer,
        extension -> Text,
    }
}

table! {
    files (id) {
        id -> Integer,
        full_path -> Text,
    }
}

table! {
    good_words (id) {
        id -> Integer,
        word -> Text,
    }
}

table! {
    ignored (id) {
        id -> Integer,
        word -> Text,
    }
}

table! {
    ignored_for_ext (id) {
        id -> Integer,
        word -> Text,
        extension_id -> Integer,
    }
}

table! {
    ignored_for_file (id) {
        id -> Integer,
        word -> Text,
        file_id -> Integer,
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
        full_path -> Text,
    }
}

joinable!(ignored_for_ext -> extensions (extension_id));
joinable!(ignored_for_file -> files (file_id));

allow_tables_to_appear_in_same_query!(
    extensions,
    files,
    good_words,
    ignored,
    ignored_for_ext,
    ignored_for_file,
    skipped_file_names,
    skipped_paths,
);
