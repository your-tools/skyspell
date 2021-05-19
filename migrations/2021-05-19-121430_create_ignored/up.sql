CREATE TABLE ignored (
  id INTEGER PRIMARY KEY NOT NULL,
  word TEXT NOT NULL
);

CREATE UNIQUE INDEX ignored_index ON ignored(word);
