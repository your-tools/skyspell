use tower_lsp::lsp_types::*;

fn get_code_action_provider_capabilities() -> Option<CodeActionProviderCapability> {
    Some(CodeActionProviderCapability::Options(CodeActionOptions {
        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
        work_done_progress_options: WorkDoneProgressOptions {
            work_done_progress: Some(false),
        },
        resolve_provider: None,
    }))
}

fn get_workspace_server_capabilities() -> Option<WorkspaceServerCapabilities> {
    Some(WorkspaceServerCapabilities {
        workspace_folders: Some(WorkspaceFoldersServerCapabilities {
            supported: Some(true),
            change_notifications: Some(OneOf::Left(true)),
        }),
        ..Default::default()
    })
}

pub(crate) fn get_capabilities() -> ServerCapabilities {
    let position_encoding = Some(PositionEncodingKind::UTF16);

    let text_document_sync = Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL));

    let code_action_provider = get_code_action_provider_capabilities();

    let workspace = get_workspace_server_capabilities();

    ServerCapabilities {
        position_encoding,
        text_document_sync,
        code_action_provider,
        workspace,
        ..Default::default()
    }
}
