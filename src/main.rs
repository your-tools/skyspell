use anyhow::{anyhow, Result};

fn main() -> Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow!("DATABASE_URL environment variable not set"))?;

    let db = rcspell::db::new(&db_url)?;
    dbg!(&db);
    Ok(())
}
