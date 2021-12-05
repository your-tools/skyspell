use std::path::Path;

use anyhow::{Context, Result};
use ignore::Walk;
use skyspell::NonInteractiveChecker;
use skyspell_aspell::AspellDictionary;
use skyspell_core::{Checker, Project, ProjectPath};
use skyspell_core::{Dictionary, TokenProcessor};
use skyspell_enchant::EnchantDictionary;

use crate::config::Config;
use crate::{parse_config, CONFIG_FILE_NAME};

pub fn main() -> Result<()> {
    println!("Checking project for spelling errors");
    let config_path = Path::new(CONFIG_FILE_NAME);
    let config = parse_config(config_path)?;

    let lang = config.lang();
    let provider = config.provider();
    if provider == "aspell" {
        // This is saves a round-trip through C++ Enchant code :)
        let dictionary = AspellDictionary::new(lang)?;
        println!("Dictionary: aspell {}", lang);
        run_ci_with(dictionary, config)
    } else {
        let dictionary = EnchantDictionary::new(lang)?;
        let current_provider = dictionary.provider();
        println!(
            "Dictionary: enchant with provider {} for {}",
            current_provider, lang
        );
        if current_provider != provider {
            eprintln!(
                "Warning: current provider does not match the one from the configuration : {}",
                provider
            );
        }
        run_ci_with(dictionary, config)
    }
}

fn run_ci_with<D: Dictionary>(dictionary: D, config: Config) -> Result<()> {
    // TODO
    let project_id = 42;
    let project_path = ProjectPath::new(Path::new("."))?;
    let project = Project::new(project_id, project_path);
    let mut checker = NonInteractiveChecker::new(project, dictionary, config)?;
    let mut num_checked = 0;

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
        num_checked += 1;
        let token_processor = TokenProcessor::new(path);
        token_processor.each_token(|token, line, column| {
            checker.handle_token(token, &relative_path, &(line, column))
        })?;
    }

    println!("Checked {} files", num_checked);
    checker.success()
}
