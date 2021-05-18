use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

use anyhow::{anyhow, Context, Result};
use platform_dirs::AppDirs;

fn main() -> Result<()> {
    let app_dirs = AppDirs::new(Some("rcspell"), false).unwrap();
    let data_dir = app_dirs.data_dir;
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("Could not create {}", data_dir.display()))?;

    let known_words: HashSet<_> = include_str!("words_en.txt")
        .split_ascii_whitespace()
        .collect();

    let db_path = &data_dir.join("en.db");
    let db_path = db_path
        .to_str()
        .ok_or_else(|| anyhow!("{} contains non-UTF-8 chars", db_path.display()))?;
    let _db = rcspell::db::new(db_path)?;

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
