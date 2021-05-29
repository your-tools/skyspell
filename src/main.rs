use anyhow::Result;
use kak_spell::cli;
use kak_spell::kak;

fn from_kak() -> bool {
    for (key, _) in std::env::vars() {
        if key.starts_with("kak_") {
            return true;
        }
    }
    false
}

fn main() -> Result<()> {
    if from_kak() {
        kak::run()
    } else {
        cli::run()
    }
}
