CREATE TABLE extensions (
  extension TEXT NOT NULL PRIMARY KEY,
  programming_language_id INTEGER NOT NULL,
  FOREIGN KEY(programming_language_id) REFERENCES programming_languages(id)
)
