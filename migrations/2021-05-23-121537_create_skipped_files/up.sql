CREATE TABLE skipped_files (
  id INTEGER PRIMARY KEY NOT NULL,
  file_name TEXT NOT NULL
);

CREATE UNIQUE INDEX skipped_file_names ON skipped_files(file_name);
