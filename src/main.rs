use anyhow::Result;
use kak_spell::cli;
use kak_spell::kak;

// The behavior of kak-spell is so different when invoked from
// kakoune and from the command line than it's best to
// have completely different main() functions
fn main() -> Result<()> {
    if from_kak() {
        kak::run()
    } else {
        cli::run()
    }
}

fn from_kak() -> bool {
    // Assume that if there's an environment variable starting with
    // `kak_`, we are running from kakoune
    for (key, _) in std::env::vars() {
        if key.starts_with("kak_") {
            return true;
        }
    }
    false
}
