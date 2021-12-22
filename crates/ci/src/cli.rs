use skyspell_core::Interactor;
use std::collections::HashSet;
use std::path::Path;

use anyhow::{bail, Context, Result};
use clap::Parser;
use ignore::Walk;
use skyspell::NonInteractiveChecker;
use skyspell_aspell::AspellDictionary;
use skyspell_core::{Checker, Project, ProjectPath, RelativePath};
use skyspell_core::{ConsoleInteractor, Repository};
use skyspell_core::{Dictionary, TokenProcessor};
use skyspell_enchant::EnchantDictionary;

use crate::config::Config;
use crate::PROJECT_ID;
use crate::{parse_config, CONFIG_FILE_NAME};

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
    let interactor = ConsoleInteractor;
    let lang = opts.lang.as_deref().unwrap_or("en_US");
    let provider = opts.provider.as_deref().unwrap_or("aspell");
    let config_path = Path::new(CONFIG_FILE_NAME);
    let config = if config_path.exists() {
        parse_config(config_path)?
    } else {
        Config::default_for(lang, provider)
    };
    match provider {
        "aspell" => {
            let dictionary = AspellDictionary::new(lang)?;
            init_with(config, dictionary, interactor)
        }
        "enchant" => {
            let dictionary = EnchantDictionary::new(lang)?;
            init_with(config, dictionary, interactor)
        }
        other => {
            bail!("no such provider {}", other)
        }
    }
}

struct InitChecker<D: Dictionary> {
    dictionary: D,
    config: Config,
    project: Project,
    interactor: ConsoleInteractor,
    skipped: HashSet<String>,
    quittting: bool,
}

impl<D: Dictionary> Checker for InitChecker<D> {
    type Context = (usize, usize);

    fn handle_error(
        &mut self,
        error: &str,
        path: &skyspell_core::RelativePath,
        context: &Self::Context,
    ) -> Result<()> {
        let &(line, column) = context;
        // The list of skipped paths may have changed
        if self.should_skip(path)? {
            return Ok(());
        }
        // We may have decided to skip this error by pressing 'x'
        if self.skipped.contains(error) {
            return Ok(());
        }
        self.on_error(path, (line, column), error)
    }

    fn success(&self) -> Result<()> {
        Ok(())
    }

    fn ignore_store(&self) -> &dyn skyspell_core::IgnoreStore {
        &self.config
    }

    fn dictionary(&self) -> &dyn Dictionary {
        &self.dictionary
    }

    fn project(&self) -> &Project {
        &self.project
    }
}

impl<D: Dictionary> InitChecker<D> {
    fn new(project: Project, interactor: ConsoleInteractor, dictionary: D, config: Config) -> Self {
        Self {
            skipped: HashSet::new(),
            quittting: false,
            project,
            interactor,
            dictionary,
            config,
        }
    }

    fn dump_config(self) -> Result<()> {
        println!("Writing config ...");
        let as_str = serde_yaml::to_string(&self.config)?;
        let config_path = Path::new(CONFIG_FILE_NAME);
        std::fs::write(&config_path, as_str)?;
        println!("done");
        Ok(())
    }

    fn on_error(&mut self, path: &RelativePath, pos: (usize, usize), error: &str) -> Result<()> {
        let (lineno, column) = pos;
        let prefix = format!("{}:{}:{}", path, lineno, column);
        println!("{} {}", prefix, error);
        let prompt = r#"What to do?
a : Add word to global ignore list
e : Add word to ignore list for this extension
f : Add word to ignore list for the current file
n : Always skip this file name
s : Always skip this file path
x : Skip this error
q : Quit
> "#;

        loop {
            let letter = self.interactor.input_letter(prompt, "aefnsxq");
            match letter.as_ref() {
                "a" => {
                    if self.on_global_ignore(error)? {
                        break;
                    }
                }
                "e" => {
                    if self.on_extension(path, error)? {
                        break;
                    }
                }
                "f" => {
                    if self.on_file_ignore(error, path)? {
                        break;
                    }
                }
                "n" => {
                    if self.on_file_name_skip(path)? {
                        break;
                    }
                }
                "s" => {
                    if self.on_project_file_skip(path)? {
                        break;
                    }
                }
                "q" => {
                    self.on_quit()?;
                    break;
                }
                "x" => {
                    self.skipped.insert(error.to_string());
                    break;
                }
                _ => {
                    unreachable!()
                }
            }
        }
        Ok(())
    }

    fn on_quit(&mut self) -> Result<bool> {
        self.quittting = true;
        Ok(true)
    }

    fn on_global_ignore(&mut self, error: &str) -> Result<bool> {
        self.config.ignore_for_project(error, self.project.id())?;
        println!(
            "Added '{}' to the ignore list for the current project",
            error
        );
        Ok(true)
    }

    fn on_extension(&mut self, relative_path: &RelativePath, error: &str) -> Result<bool> {
        let extension = match relative_path.extension() {
            None => {
                eprintln!("{} has no extension", relative_path);
                return Ok(false);
            }
            Some(e) => e,
        };

        self.config.ignore_for_extension(error, &extension)?;
        println!(
            "Added '{}' to the ignore list for extension '{}'",
            error, extension
        );
        Ok(true)
    }

    fn on_file_ignore(&mut self, error: &str, relative_path: &RelativePath) -> Result<bool> {
        self.config
            .ignore_for_path(error, self.project.id(), relative_path)?;
        println!(
            "Added '{}' to the ignore list for path '{}'",
            error, relative_path
        );
        Ok(true)
    }

    fn on_file_name_skip(&mut self, relative_path: &RelativePath) -> Result<bool> {
        let file_name = match relative_path.file_name() {
            None => {
                eprintln!("{} has no file name", relative_path);
                return Ok(false);
            }
            Some(r) => r,
        };

        self.config.skip_file_name(&file_name)?;

        println!("Added '{}' to the list of file names to skip", file_name);
        Ok(true)
    }

    fn on_project_file_skip(&mut self, relative_path: &RelativePath) -> Result<bool> {
        self.config.skip_path(self.project().id(), relative_path)?;
        println!(
            "Added '{}' to the list of files to skip for the current project",
            relative_path,
        );
        Ok(true)
    }
}

fn init_with<D: Dictionary>(
    config: Config,
    dictionary: D,
    interactor: ConsoleInteractor,
) -> Result<()> {
    let project_path = ProjectPath::new(Path::new("."))?;
    let project = Project::new(PROJECT_ID, project_path);
    let mut checker = InitChecker::new(project, interactor, dictionary, config);
    for result in Walk::new("./") {
        if checker.quittting {
            break;
        }
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
            if checker.quittting {
                return Ok(());
            }
            checker.handle_token(token, &relative_path, &(line, column))
        })?;
    }

    checker.success()?;

    checker.dump_config()
}
