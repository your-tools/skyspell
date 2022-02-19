CREATE TABLE skipped_file_names (
  id INTEGER PRIMARY KEY NOT NULL,
  file_name TEXT NOT NULL
);

CREATE UNIQUE INDEX skipped_file_names_index ON skipped_file_names(file_name);

CREATE TABLE skipped_paths (
  id INTEGER PRIMARY KEY NOT NULL,
  project_id INTEGER NOT NULL,
  path TEXT NOT NULL,
  FOREIGN KEY(project_id) REFERENCES project(id)
  UNIQUE(project_id, path)
);
