use std::collections::hash_map::Entry;
use std::sync::Arc;

use base::rwlock::RwLock;
use itertools::Itertools;

use crate::asm::handler::AssemblyLanguageServerProtocol;
use crate::diagnostics::{Error, UrlPath};

use super::context::Context;
use super::error::{lsp_error_map, ErrorCode};
use super::ext::{FileStatsParams, FileStatsResult};
use super::types::{
    CodeActionMessage, DocumentChange, DocumentRangeMessage, FindReferencesMessage,
    LocationMessage, RenameMessage,
};

use lsp_server::ResponseError;
use lsp_types::{
    CompletionList, DidChangeTextDocumentParams, SignatureHelp, TextEdit, Url, WorkspaceEdit,
};

pub fn open_file(
    context: Arc<Context>,
    lang_id: &str,
    url: Url,
    text: &str,
    version: u32,
) -> Result<(), ResponseError> {
    let actor = match lang_id.to_lowercase().as_str() {
        "asm" => AssemblyLanguageServerProtocol::new(context.clone(), text, url.clone(), version),
        "assembly" => {
            AssemblyLanguageServerProtocol::new(context.clone(), text, url.clone(), version)
        }
        _ => panic!("Unknown language: {lang_id}"),
    };

    context.actors.write().insert(url, RwLock::new(actor));
    Ok(())
}

pub fn update_file(
    context: Arc<Context>,
    msg: DidChangeTextDocumentParams,
) -> Result<(), ResponseError> {
    let uri = msg.text_document.uri;
    let mut change_params = msg.content_changes;

    let changes = change_params
        .drain(..)
        .map(|c| DocumentChange {
            text: c.text,
            range: c.range.map(|r| r.into()),
        })
        .collect();

    let new_actors = {
        let actors = context.actors.read();
        let mut actor = actors
            .get(&uri)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .write();

        actor.update(context.clone(), msg.text_document.version as _, changes)?
    };

    context.add_actors(new_actors);

    Ok(())
}

pub fn close_file(context: Arc<Context>, url: Url) -> Result<(), ResponseError> {
    // Only close the file if nothing else has a reference to it.
    if !context.file_graph.read().has_references(url.as_ref()) {
        if let Entry::Occupied(entry) = context.actors.write().entry(url) {
            entry.remove_entry();
        }
    }
    Ok(())
}

pub fn goto_definition(
    context: Arc<Context>,
    request: LocationMessage,
) -> Result<lsp_types::GotoDefinitionResponse, ResponseError> {
    context
        .actors
        .read()
        .get(&request.url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .goto_definition(context.clone(), request.position)
}

pub fn find_references(
    context: Arc<Context>,
    request: FindReferencesMessage,
) -> Result<Vec<lsp_types::Location>, ResponseError> {
    context
        .actors
        .read()
        .get(&request.location.url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .find_references(
            context.clone(),
            request.location.position,
            request.include_decl,
        )
}

pub fn hover(
    context: Arc<Context>,
    request: LocationMessage,
) -> Result<Option<lsp_types::Hover>, ResponseError> {
    context
        .actors
        .read()
        .get(&request.url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .hover(context.clone(), request.position)
}

pub fn code_action(
    context: Arc<Context>,
    request: CodeActionMessage,
) -> Result<lsp_types::CodeActionResponse, ResponseError> {
    context
        .actors
        .read()
        .get(&request.url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .code_actions(context.clone(), request.range)
}

pub fn document_highlight(
    context: Arc<Context>,
    request: LocationMessage,
) -> Result<Vec<lsp_types::DocumentHighlight>, ResponseError> {
    context
        .actors
        .read()
        .get(&request.url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .document_highlight(context.clone(), request.position)
}

pub fn get_semantic_tokens(
    context: Arc<Context>,
    request: DocumentRangeMessage,
) -> Result<lsp_types::SemanticTokensResult, ResponseError> {
    context
        .actors
        .read()
        .get(&request.url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .get_semantic_tokens(context.clone(), request.range.map(|r| r.into()))
}

pub fn document_symbols(
    context: Arc<Context>,
    url: Url,
) -> Result<lsp_types::DocumentSymbolResponse, ResponseError> {
    context
        .actors
        .read()
        .get(&url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .document_symbols(context.clone())
}

pub fn code_lens(
    context: Arc<Context>,
    url: Url,
) -> Result<Option<Vec<lsp_types::CodeLens>>, ResponseError> {
    context
        .actors
        .read()
        .get(&url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .code_lens(context.clone())
}

pub fn completion(
    context: Arc<Context>,
    location: LocationMessage,
) -> Result<CompletionList, ResponseError> {
    context
        .actors
        .read()
        .get(&location.url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .completion(context.clone(), location.position)
}

pub fn signature_help(
    context: Arc<Context>,
    request: &LocationMessage,
) -> Result<Option<SignatureHelp>, ResponseError> {
    context
        .actors
        .read()
        .get(&request.url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .signature_help(context.clone(), &request.position)
}

pub fn get_diagnostics(context: Arc<Context>, uri: &Url) -> Option<Vec<Error>> {
    Some(
        context
            .commands
            .as_ref()?
            .assembler_for_file(uri)?
            .get_errors()
            .into_iter()
            .filter(|err| {
                uri.try_into()
                    .map(|uri: UrlPath| uri.is_file(&err.file))
                    .unwrap_or(false)
            })
            .collect_vec(),
    )
}

pub fn format(context: Arc<Context>, url: Url) -> Result<Option<Vec<TextEdit>>, ResponseError> {
    context
        .actors
        .read()
        .get(&url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .format(context.clone(), &context.root)
}

pub fn rename(
    context: Arc<Context>,
    rename: RenameMessage,
) -> Result<Option<WorkspaceEdit>, ResponseError> {
    context
        .actors
        .read()
        .get(&rename.location.url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .rename(context.clone(), rename)
}

pub fn syntax_tree(context: Arc<Context>, url: Url) -> Result<String, ResponseError> {
    context
        .actors
        .read()
        .get(&url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .syntax_tree()
}

pub fn analysis(
    context: Arc<Context>,
    request: DocumentRangeMessage,
) -> Result<String, ResponseError> {
    context
        .actors
        .read()
        .get(&request.url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read()
        .analysis(context.clone(), request.range)
}

pub fn file_stats(
    context: Arc<Context>,
    request: FileStatsParams,
) -> Result<FileStatsResult, ResponseError> {
    let actors = context.actors.read();
    let actor = actors
        .get(&request.url)
        .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
        .read();

    Ok(FileStatsResult {
        version: actor.version(),
    })
}
