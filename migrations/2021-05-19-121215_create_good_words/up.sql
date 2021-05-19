CREATE TABLE good_words (
  id INTEGER PRIMARY KEY NOT NULL,
  word TEXT NOT NULL
);

CREATE UNIQUE INDEX good_words_index ON good_words(word);
