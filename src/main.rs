use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use platform_dirs::AppDirs;

use rcspell::Repo;

fn main() -> Result<()> {
    let app_dirs = AppDirs::new(Some("rcspell"), false).unwrap();
    let data_dir = app_dirs.data_dir;
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("Could not create {}", data_dir.display()))?;

    let db_path = &data_dir.join("en.db");
    let db_path = db_path
        .to_str()
        .ok_or_else(|| anyhow!("{} contains non-UTF-8 chars", db_path.display()))?;
    let mut db = rcspell::db::new(db_path)?;
    if !db.has_good_words()? {
        let known_words: Vec<_> = include_str!("words_en.txt")
            .split_ascii_whitespace()
            .collect();
        db.add_good_words(&known_words)?;
    }

    let interactor = rcspell::ConsoleInteractor;
    let mut handler = rcspell::Handler::new(interactor, db);

    let args: Vec<_> = std::env::args().collect();
    let source_path = PathBuf::from(&args[1]);
    let source = File::open(&source_path)?;
    let reader = BufReader::new(source);

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let tokenizer = rcspell::Tokenizer::new(&line);
        for (word, pos) in tokenizer {
            let is_known = handler.handle_token(&source_path, word)?;
            if !is_known {
                handler.handle_error(&source_path, (i + 1, pos), word)?;
            }
        }
    }

    Ok(())
}
