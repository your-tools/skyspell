CREATE TABLE ignored_for_ext (
  id INTEGER PRIMARY KEY NOT NULL,
  word TEXT NOT NULL,
  extension_id INTEGER NOT NULL,
  FOREIGN KEY(extension_id) REFERENCES extensions(id)
);
