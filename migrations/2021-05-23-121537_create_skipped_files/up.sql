CREATE TABLE skipped_paths (
  id INTEGER PRIMARY KEY NOT NULL,
  full_path TEXT NOT NULL
);

CREATE UNIQUE INDEX skipped_paths_index ON skipped_paths(full_path);

CREATE TABLE skipped_file_names (
  id INTEGER PRIMARY KEY NOT NULL,
  file_name TEXT NOT NULL
);

CREATE UNIQUE INDEX skipped_file_names_index ON skipped_file_names(file_name);
