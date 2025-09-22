use std::sync::Mutex;

use skyspell_core::{Checker, SystemDictionary};
use tower_lsp::lsp_types::*;

struct LspChecker {
    dict: SystemDictionary,
}

pub struct State {
    workspace_folders: Vec<WorkspaceFolder>,
    checkers: Vec<LspChecker>,
}
impl State {
    pub(crate) fn new() -> Self {
        Self {
            workspace_folders: vec![],
            checkers: vec![],
        }
    }

    pub(crate) fn set_workspace_folders(&mut self, folders: Vec<WorkspaceFolder>) {
        self.workspace_folders = folders;
        self.display_workspaces();
    }

    pub(crate) fn update_workspace_folders(&mut self, params: DidChangeWorkspaceFoldersParams) {
        let WorkspaceFoldersChangeEvent { added, removed } = params.event;
        self.workspace_folders.extend(added);
        if !removed.is_empty() {
            self.workspace_folders.retain(|x| !removed.contains(x));
        }
    }

    fn display_workspaces(&mut self) {
        let names: Vec<String> = self
            .workspace_folders
            .iter()
            .map(|f| f.name.clone())
            .collect();
    }
}
