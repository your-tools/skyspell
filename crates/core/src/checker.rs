use crate::{Dictionary, IgnoreStore, Operation, TokenProcessor};
use crate::{Project, ProjectFile};
use anyhow::{Context, Result, anyhow, bail};
use directories_next::BaseDirs;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub struct SpellingError {
    pub word: String,
    pub project_file: ProjectFile,
    pub pos: (usize, usize),
}

impl SpellingError {
    pub fn new(word: String, pos: (usize, usize), project_file: &ProjectFile) -> Self {
        Self {
            word,
            pos,
            project_file: project_file.clone(),
        }
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn project_file(&self) -> &ProjectFile {
        &self.project_file
    }

    pub fn pos(&self) -> (usize, usize) {
        self.pos
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessOutcome {
    Skipped,
    Checked,
}

pub trait Checker<D: Dictionary> {
    type SourceContext;

    fn dictionary(&self) -> &D;

    fn project(&self) -> &Project;

    #[deprecated]
    fn to_project_file(&self, path: &Path) -> Result<ProjectFile> {
        ProjectFile::new(self.project(), path)
    }

    // Were all the errors handled properly?
    fn success(&self) -> Result<()>;

    fn ignore_store(&mut self) -> &mut IgnoreStore;

    fn state(&mut self) -> Option<&mut CheckerState> {
        None
    }

    fn process(
        &mut self,
        source_path: &Path,
        context: &Self::SourceContext,
    ) -> Result<ProcessOutcome> {
        let skip_file = self.project().skip_file();
        let project_file = ProjectFile::new(self.project(), source_path)?;
        if skip_file.is_skipped(&project_file) {
            return Ok(ProcessOutcome::Skipped);
        }
        let file = File::open(source_path)?;
        let reader = BufReader::new(&file);
        let file_name = source_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let mut token_processor = TokenProcessor::new(reader, &file_name);
        let skipped_tokens = self.ignore_store().skipped_tokens(&project_file);
        token_processor.skip_tokens(&skipped_tokens);
        for token in token_processor {
            let token = token?;
            self.handle_token(&token.text, &project_file, token.pos, context)?;
        }
        Ok(ProcessOutcome::Checked)
    }

    fn handle_error(&mut self, error: &SpellingError, context: &Self::SourceContext) -> Result<()>;

    fn handle_token(
        &mut self,
        token: &str,
        project_file: &ProjectFile,
        pos: (usize, usize),
        context: &Self::SourceContext,
    ) -> Result<()> {
        let dictionary = self.dictionary();
        let lang = dictionary.lang().to_owned();
        let in_dict = dictionary.check(token)?;
        if in_dict {
            return Ok(());
        }
        let should_ignore = self
            .ignore_store()
            .should_ignore(token, project_file, &lang);
        if should_ignore {
            return Ok(());
        }
        let error = SpellingError::new(token.to_owned(), pos, project_file);
        self.handle_error(&error, context)?;
        Ok(())
    }

    fn apply_operation(&mut self, mut operation: Operation) -> Result<()> {
        let store = self.ignore_store();
        operation.execute(store)?;
        if let Some(state) = self.state() {
            state.set_last_operation(operation.clone())?;
        }
        Ok(())
    }

    fn undo(&mut self) -> Result<()> {
        let state = match self.state() {
            None => bail!("Cannot undo"),
            Some(s) => s,
        };
        let last_operation = state.pop_last_operation()?;
        let mut last_operation = match last_operation {
            None => bail!("Nothing to undo"),
            Some(o) => o,
        };
        let store = self.ignore_store();
        last_operation.undo(store)
    }
}

pub struct CheckerState {
    storage_path: PathBuf,
    inner: StateInner,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct StateInner {
    last_operation: Option<Operation>,
}

impl CheckerState {
    pub fn load(state_toml: Option<PathBuf>) -> Result<Self> {
        let state_toml = match state_toml {
            None => {
                let base_dirs =
                    BaseDirs::new().ok_or_else(|| anyhow!("Could not get home directory"))?;
                let data_dir = base_dirs.data_dir().join("skyspell");
                std::fs::create_dir_all(&data_dir)
                    .with_context(|| format!("Could not create data dir {}", data_dir.display()))?;
                data_dir.join("state.toml")
            }
            Some(p) => p,
        };
        let inner: StateInner = if state_toml.exists() {
            let contents = std::fs::read_to_string(&state_toml)
                .with_context(|| format!("Could not read from {}", state_toml.display()))?;
            toml::from_str(&contents)
                .with_context(|| format!("Could not parse {}", state_toml.display()))?
        } else {
            Default::default()
        };

        Ok(CheckerState {
            storage_path: state_toml,
            inner,
        })
    }

    pub fn set_last_operation(&mut self, operation: Operation) -> Result<()> {
        self.inner.last_operation = Some(operation);
        self.save()
    }

    pub fn pop_last_operation(&mut self) -> Result<Option<Operation>> {
        let result = self.inner.last_operation.take();
        self.save()?;
        Ok(result)
    }

    fn save(&self) -> Result<()> {
        let contents = toml_edit::ser::to_string_pretty(&self.inner)
            .with_context(|| "Could not serialize state")?;
        std::fs::write(&self.storage_path, contents)
            .with_context(|| "Could not write to storage path")?;
        Ok(())
    }
}
