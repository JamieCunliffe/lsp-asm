use std::collections::hash_map::Entry;
use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::asm::handler::AssemblyLanguageServerProtocol;
use crate::config::LSPConfig;
use crate::diagnostics::assembler_flags::AssemblerFlags;
use crate::diagnostics::compile_commands::CompileCommands;
use crate::diagnostics::{Diagnostics, Error};

use super::error::{lsp_error_map, ErrorCode};
use super::types::{DocumentChange, DocumentRangeMessage, FindReferencesMessage, LocationMessage};
use super::LanguageServerProtocol;

use lsp_server::ResponseError;
use lsp_types::{CompletionList, DidChangeTextDocumentParams, SignatureHelp, Url};

pub struct LangServerHandler {
    actors: RwLock<HashMap<Url, RwLock<Box<dyn LanguageServerProtocol + Send + Sync>>>>,
    config: LSPConfig,
    commands: Option<Box<dyn Diagnostics + Send + Sync>>,
}

impl LangServerHandler {
    pub fn new(config: LSPConfig, root: String) -> Self {
        info!("Initializing workspace: {}", root);

        let commands: Option<Box<dyn Diagnostics + Send + Sync>> =
            if let Some(compile_commands) = CompileCommands::new(&root) {
                Some(Box::new(compile_commands))
            } else if let Some(flags) = AssemblerFlags::new(&root) {
                Some(Box::new(flags))
            } else {
                None
            };

        Self {
            actors: RwLock::new(HashMap::new()),
            config,
            commands,
        }
    }

    pub fn config(&self) -> &LSPConfig {
        &self.config
    }

    pub async fn open_file(
        &self,
        lang_id: &str,
        url: Url,
        text: &str,
        version: u32,
    ) -> Result<(), ResponseError> {
        let actor = match lang_id.to_lowercase().as_str() {
            "asm" => {
                AssemblyLanguageServerProtocol::new(text, url.clone(), version, self.config.clone())
            }
            "assembly" => {
                AssemblyLanguageServerProtocol::new(text, url.clone(), version, self.config.clone())
            }
            _ => panic!("Unknown language: {}", lang_id),
        };

        self.actors
            .write()
            .await
            .insert(url, RwLock::new(Box::new(actor)));

        Ok(())
    }

    pub async fn update_file(&self, msg: DidChangeTextDocumentParams) -> Result<(), ResponseError> {
        let uri = msg.text_document.uri;
        let mut change_params = msg.content_changes;

        let changes = change_params
            .drain(..)
            .map(|c| DocumentChange {
                text: c.text,
                range: c.range.map(|r| r.into()),
            })
            .collect();

        if self
            .actors
            .read()
            .await
            .get(&uri)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .write()
            .await
            .update(msg.text_document.version as _, changes)
        {
            Ok(())
        } else {
            warn!("Out of order document updates, request full sync");
            Err(lsp_error_map(ErrorCode::InvalidVersion(uri)))
        }
    }

    pub async fn close_file(&self, url: Url) -> Result<(), ResponseError> {
        if let Entry::Occupied(entry) = self.actors.write().await.entry(url) {
            entry.remove_entry();
        }

        Ok(())
    }

    pub async fn goto_definition(
        &self,
        request: LocationMessage,
    ) -> Result<lsp_types::GotoDefinitionResponse, ResponseError> {
        self.actors
            .read()
            .await
            .get(&request.url)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .read()
            .await
            .goto_definition(request.position)
    }

    pub async fn find_references(
        &self,
        request: FindReferencesMessage,
    ) -> Result<Vec<lsp_types::Location>, ResponseError> {
        self.actors
            .read()
            .await
            .get(&request.location.url)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .read()
            .await
            .find_references(request.location.position, request.include_decl)
    }

    pub async fn hover(
        &self,
        request: LocationMessage,
    ) -> Result<Option<lsp_types::Hover>, ResponseError> {
        self.actors
            .read()
            .await
            .get(&request.url)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .read()
            .await
            .hover(request.position)
    }

    pub async fn document_highlight(
        &self,
        request: LocationMessage,
    ) -> Result<Vec<lsp_types::DocumentHighlight>, ResponseError> {
        self.actors
            .read()
            .await
            .get(&request.url)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .read()
            .await
            .document_highlight(request.position)
    }

    pub async fn get_semantic_tokens(
        &self,
        request: DocumentRangeMessage,
    ) -> Result<lsp_types::SemanticTokensResult, ResponseError> {
        self.actors
            .read()
            .await
            .get(&request.url)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .read()
            .await
            .get_semantic_tokens(request.range.map(|r| r.into()))
    }

    pub async fn document_symbols(
        &self,
        url: Url,
    ) -> Result<lsp_types::DocumentSymbolResponse, ResponseError> {
        self.actors
            .read()
            .await
            .get(&url)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .read()
            .await
            .document_symbols()
    }

    pub async fn code_lens(
        &self,
        url: Url,
    ) -> Result<Option<Vec<lsp_types::CodeLens>>, ResponseError> {
        self.actors
            .read()
            .await
            .get(&url)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .read()
            .await
            .code_lens()
    }

    pub async fn completion(
        &self,
        location: LocationMessage,
    ) -> Result<CompletionList, ResponseError> {
        self.actors
            .read()
            .await
            .get(&location.url)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .read()
            .await
            .completion(location.position)
    }

    pub async fn signature_help(
        &self,
        request: &LocationMessage,
    ) -> Result<Option<SignatureHelp>, ResponseError> {
        self.actors
            .read()
            .await
            .get(&request.url)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .read()
            .await
            .signature_help(&request.position)
    }

    pub async fn get_diagnostics(&self, uri: &Url) -> Option<Vec<Error>> {
        Some(
            self.commands
                .as_ref()?
                .assembler_for_file(uri)?
                .get_errors(),
        )
    }

    pub async fn syntax_tree(&self, url: Url) -> Result<String, ResponseError> {
        self.actors
            .read()
            .await
            .get(&url)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .read()
            .await
            .syntax_tree()
    }

    pub async fn analysis(&self, request: DocumentRangeMessage) -> Result<String, ResponseError> {
        self.actors
            .read()
            .await
            .get(&request.url)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .read()
            .await
            .analysis(request.range)
    }
}
