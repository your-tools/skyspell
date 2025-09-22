use std::sync::{Arc, Mutex};

use tower_lsp::jsonrpc::{self, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::State;
use crate::capabilities::get_capabilities;
use crate::server_info::get_server_info;

pub struct Backend {
    client: Client,
    state: Arc<Mutex<State>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        let state = State::new();
        Self {
            client,
            state: Mutex::new(state),
        }
    }
}

impl Backend {
    async fn log_info(&self, message: &str) {
        self.client.log_message(MessageType::INFO, message).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let capabilities = get_capabilities();
        let server_info = get_server_info();
        let mut state = self.state.lock().unwrap();

        state.set_workspace_folders(params.workspace_folders.unwrap_or_default());

        Ok(InitializeResult {
            capabilities,
            server_info,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.log_info("server initialized").await
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        self.log_info(&format!("did open {uri}")).await;
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        let mut state = self.state.lock().unwrap();
        state.update_workspace_folders(params);
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        self.log_info("did save").await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.log_info("did close").await;

        // clear diagnostics to avoid a stale diagnostics flash on open
        // if the file has typos fixed outside of vscode
        // see https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let params = serde_json::to_string(&params);
        self.log_info(&format!("did change: {params:?}")).await;
    }

    async fn code_action(
        &self,
        _params: CodeActionParams,
    ) -> jsonrpc::Result<Option<CodeActionResponse>> {
        Ok(Some(vec![]))
    }
}
