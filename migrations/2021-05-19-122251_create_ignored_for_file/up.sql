CREATE TABLE ignored_for_file (
  id INTEGER PRIMARY KEY NOT NULL,
  word TEXT NOT NULL,
  file_id INTEGER NOT NULL,
  FOREIGN KEY(file_id) REFERENCES files(id)
);
