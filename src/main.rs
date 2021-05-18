use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

use anyhow::{anyhow, Result};

fn main() -> Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow!("DATABASE_URL environment variable not set"))?;

    let _db = rcspell::db::new(&db_url)?;
    let known_words: HashSet<_> = include_str!("words_en.txt")
        .split_ascii_whitespace()
        .collect();

    let args: Vec<_> = std::env::args().collect();
    let source_path = &args[1];
    let source = File::open(&source_path)?;
    let reader = BufReader::new(source);

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let tokenizer = rcspell::Tokenizer::new(&line);
        for (word, pos) in tokenizer {
            if !known_words.contains(word) {
                println!("{}{}:{} {}", source_path, i + 1, pos, word);
            }
        }
    }

    Ok(())
}
