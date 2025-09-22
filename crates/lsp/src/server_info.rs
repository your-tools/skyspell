use tower_lsp::lsp_types::*;

pub(crate) fn get_server_info() -> Option<ServerInfo> {
    Some(ServerInfo {
        name: "skyspell".to_string(),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
    })
}
