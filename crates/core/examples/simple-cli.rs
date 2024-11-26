use std::path::Path;

use anyhow::{bail, Result};
use skyspell_core::{Checker, IgnoreStore, SpellingError};
use skyspell_core::{Project, SystemDictionary};
#[cfg(target_family = "windows")]
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

struct ExampleChecker {
    dictionary: SystemDictionary,
    project: Project,
    ignore_store: IgnoreStore,
    error_count: usize,
}

impl ExampleChecker {
    fn try_new() -> Result<Self> {
        // This must match a dictionary installed on your operating system.
        let dictionary = SystemDictionary::new("en_US")?;
        let project = Project::new(Path::new("."))?;
        let ignore_store = project.ignore_store()?;
        Ok(Self {
            dictionary,
            project,
            ignore_store,
            error_count: 0,
        })
    }
}

impl Checker<SystemDictionary> for ExampleChecker {
    // This can be used to give the handle_error() method additional context
    // while processing paths
    type SourceContext = ();

    // You have to implement those getter methods
    fn dictionary(&self) -> &SystemDictionary {
        &self.dictionary
    }

    fn project(&self) -> &Project {
        &self.project
    }

    fn ignore_store(&mut self) -> &mut IgnoreStore {
        &mut self.ignore_store
    }

    fn success(&self) -> Result<()> {
        if self.error_count != 0 {
            bail!("Found some errors");
        }
        Ok(())
    }

    fn handle_error(&mut self, error: &SpellingError, _context: &()) -> Result<()> {
        let (line, column) = error.pos();
        let path = error.relative_path();
        let word = error.word();
        println!("{}:{line}:{column} {word}", path.as_str());
        self.error_count += 1;
        Ok(())
    }
}

fn main() -> Result<()> {
    #[cfg(target_family = "windows")]
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
    }

    let mut checker = ExampleChecker::try_new()?;
    let source_path = Path::new("README.md");
    checker.process(source_path, &())?;
    checker.success()?;
    println!("No errors found");
    Ok(())
}
