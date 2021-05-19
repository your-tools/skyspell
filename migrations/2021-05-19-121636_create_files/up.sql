CREATE TABLE files (
  id INTEGER PRIMARY KEY NOT NULL,
  full_path TEXT NOT NULL
);

CREATE UNIQUE INDEX files_index ON files(full_path);

