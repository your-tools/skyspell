use anyhow::Result;
use clap::Clap;
use skyspell::cli::{run, Opts};
use skyspell::sql_repository::{get_default_db_path, SQLRepository};
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
    run(opts, dictionary, repository)
}
