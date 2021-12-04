use std::path::Path;

use anyhow::{Context, Result};
use ignore::Walk;
use skyspell_core::Dictionary;
use skyspell_core::TokenProcessor;
use skyspell_core::{Checker, Project, ProjectPath};
use skyspell_enchant::EnchantDictionary;

use skyspell::NonInteractiveChecker;

use crate::{parse_config, CONFIG_FILE_NAME};

pub fn main() -> Result<()> {
    let config_path = Path::new(CONFIG_FILE_NAME);
    let config = parse_config(config_path)?;

    let lang = config.enchant_lang();
    let dictionary = EnchantDictionary::new(lang)?;
    let current_provider = dictionary.provider();
    let expected_provider = config.enchant_provider();
    if current_provider != expected_provider {
        println!(
            "Warning: current Enchant provider ({}) does not match the one from the configuration : {}",
            current_provider, expected_provider);
    }

    // TODO
    let project_id = 42;
    let project_path = ProjectPath::new(Path::new("."))?;
    let project = Project::new(project_id, project_path);
    let mut checker = NonInteractiveChecker::new(project, dictionary, config)?;

    println!("Checking project for spelling errors");
    for result in Walk::new("./") {
        let entry = result.with_context(|| "Error when walking project sources")?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let relative_path = checker.to_relative_path(path)?;
        if checker.should_skip(&relative_path)? {
            println!("Skipped: {}", relative_path);
            continue;
        }
        let token_processor = TokenProcessor::new(path);
        token_processor.each_token(|token, line, column| {
            checker.handle_token(token, &relative_path, &(line, column))
        })?;
    }

    checker.success()
}
