use anyhow::Result;
use clap::Clap;
use skyspell::cli::{print_error, run, Opts};
use skyspell::sql::{get_default_db_path, SQLRepository};
use skyspell::EnchantDictionary;

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let lang = match &opts.lang {
        Some(s) => s,
        None => "en_US",
    };

    let db_path = match opts.db_path.as_ref() {
        Some(s) => Ok(s.to_string()),
        None => get_default_db_path(lang),
    }?;

    let repository = SQLRepository::new(&db_path)?;
    let mut broker = enchant::Broker::new();
    let dictionary = EnchantDictionary::new(&mut broker, lang)?;
    if let Err(e) = run(opts, dictionary, repository) {
        print_error(&e.to_string());
        std::process::exit(1);
    }
    Ok(())
}
