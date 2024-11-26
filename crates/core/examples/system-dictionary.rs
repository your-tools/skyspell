use anyhow::{bail, Result};
use skyspell_core::{Dictionary, SystemDictionary};

fn main() -> Result<()> {
    SystemDictionary::init();
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        bail!("Usage: system-dictionary <lang> <word>")
    }
    let lang = &args[1];
    let word = &args[2];
    let spell_client = SystemDictionary::new(lang)?;
    let ok = spell_client.check(word)?;
    if ok {
        println!("No error")
    } else {
        println!("Unknown word");
        let suggestions = spell_client.suggest(word)?;
        println!("Suggestions: {:?}", suggestions)
    }
    Ok(())
}
