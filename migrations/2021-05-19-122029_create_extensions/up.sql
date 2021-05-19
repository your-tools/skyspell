CREATE TABLE extensions (
  id INTEGER PRIMARY KEY NOT NULL,
  extension TEXT NOT NULL
);

CREATE UNIQUE INDEX extensions_index ON extensions(extension);

