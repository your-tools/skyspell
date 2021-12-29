use std::path::Path;

use anyhow::{bail, Context, Result};
use clap::Parser;
use ignore::Walk;
use skyspell::InteractiveChecker;
use skyspell::NonInteractiveChecker;
use skyspell_aspell::AspellDictionary;
use skyspell_core::ConsoleInteractor;
use skyspell_core::{Checker, Project, ProjectPath};
use skyspell_core::{Dictionary, TokenProcessor};
use skyspell_enchant::EnchantDictionary;
use skyspell_yaml::Config;
use skyspell_yaml::PROJECT_ID;
use skyspell_yaml::{parse_config, CONFIG_FILE_NAME};

#[derive(Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Parser)]
enum Action {
    #[clap(about = "Check all project files")]
    Run,
    #[clap(about = "Init config file")]
    Init(InitOpts),
}

#[derive(Parser)]
struct InitOpts {
    #[clap(long, about = "language")]
    lang: Option<String>,
    #[clap(long, about = "provider")]
    provider: Option<String>,
}

pub fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    match opts.action {
        Action::Init(opts) => init(opts),
        Action::Run => run(),
    }
}

fn run() -> Result<()> {
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
    let project_path = ProjectPath::new(Path::new("."))?;
    let project = Project::new(PROJECT_ID, project_path);
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

fn init(opts: InitOpts) -> Result<()> {
    let lang = opts.lang.as_deref().unwrap_or("en_US");
    let provider = opts.provider.as_deref().unwrap_or("aspell");
    let config_path = Path::new(CONFIG_FILE_NAME);
    let config = if config_path.exists() {
        parse_config(config_path)?
    } else {
        Config::init(lang, provider)
    };
    match provider {
        "aspell" => {
            let dictionary = AspellDictionary::new(lang)?;
            init_with(config, dictionary)
        }
        "enchant" => {
            let dictionary = EnchantDictionary::new(lang)?;
            init_with(config, dictionary)
        }
        other => {
            bail!("no such provider {}", other)
        }
    }
}

struct InitChecker<D: Dictionary>(InteractiveChecker<ConsoleInteractor, D, Config>);

impl<D: Dictionary> InitChecker<D> {
    fn new(project: Project, dictionary: D, config: Config) -> Result<Self> {
        let interactor = ConsoleInteractor;
        let checker = InteractiveChecker::new(project, interactor, dictionary, config)?;
        Ok(Self(checker))
    }

    fn dump_config(&mut self) -> Result<()> {
        let config = self.0.repository();
        let config_path = Path::new(CONFIG_FILE_NAME);
        config.save(config_path)
    }
}

impl<D: Dictionary> Checker for InitChecker<D> {
    type Context = <InteractiveChecker<ConsoleInteractor, D, Config> as Checker>::Context;

    fn handle_error(
        &mut self,
        error: &str,
        path: &skyspell_core::RelativePath,
        context: &Self::Context,
    ) -> Result<()> {
        self.0.handle_error(error, path, context)?;
        self.dump_config()?;
        Ok(())
    }

    fn success(&self) -> Result<()> {
        self.0.success()
    }

    fn ignore_store(&self) -> &dyn skyspell_core::IgnoreStore {
        self.0.ignore_store()
    }

    fn dictionary(&self) -> &dyn Dictionary {
        self.0.dictionary()
    }

    fn project(&self) -> &Project {
        self.0.project()
    }
}

fn init_with<D: Dictionary>(config: Config, dictionary: D) -> Result<()> {
    let project_path = ProjectPath::new(Path::new("."))?;
    let project = Project::new(PROJECT_ID, project_path);
    let mut checker = InitChecker::new(project, dictionary, config)?;
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

    checker.success()?;

    checker.dump_config()
}
