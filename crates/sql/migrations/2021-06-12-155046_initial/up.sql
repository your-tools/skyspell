CREATE TABLE ignored (
  id INTEGER PRIMARY KEY NOT NULL,
  word TEXT NOT NULL
);

CREATE UNIQUE INDEX ignored_index ON ignored(word);

CREATE TABLE ignored_for_extension (
  id INTEGER PRIMARY KEY NOT NULL,
  word TEXT NOT NULL,
  extension TEXT NOT NULL,
  UNIQUE(word, extension)
);

CREATE TABLE skipped_file_names (
  id INTEGER PRIMARY KEY NOT NULL,
  file_name TEXT NOT NULL
);

CREATE UNIQUE INDEX skipped_file_names_index ON skipped_file_names(file_name);

CREATE TABLE projects (
  id INTEGER PRIMARY KEY NOT NULL,
  path TEXT NOT NULL
);

CREATE UNIQUE INDEX project_paths_index ON projects(path);

CREATE TABLE ignored_for_project (
  id INTEGER PRIMARY KEY NOT NULL,
  word TEXT NOT NULL,
  project_id INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES project(id),
  UNIQUE(word, project_id)
);

CREATE TABLE ignored_for_path (
  id INTEGER PRIMARY KEY NOT NULL,
  word TEXT NOT NULL,
  project_id INTEGER NOT NULL,
  path TEXT NOT NULL,
  FOREIGN KEY(project_id) REFERENCES project(id),
  UNIQUE(word, path)
);

CREATE TABLE skipped_paths (
  id INTEGER PRIMARY KEY NOT NULL,
  project_id INTEGER NOT NULL,
  path TEXT NOT NULL,
  FOREIGN KEY(project_id) REFERENCES project(id)
  UNIQUE(project_id, path)
);
