CREATE TABLE ignored (
  id INTEGER PRIMARY KEY NOT NULL,
  word TEXT NOT NULL,
  file_id INTEGER,
  programming_language_id INTEGER,
  FOREIGN KEY(file_id) REFERENCES files(id),
  FOREIGN KEY(programming_language_id) REFERENCES programming_language(id)
);

CREATE INDEX ignored_index ON ignored(word);
