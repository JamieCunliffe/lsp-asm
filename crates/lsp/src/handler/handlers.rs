use std::collections::hash_map::Entry;
use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::asm::handler::AssemblyLanguageServerProtocol;
use crate::config::LSPConfig;

use super::error::{lsp_error_map, ErrorCode};
use super::types::{FindReferencesMessage, LocationMessage, SemanticTokensMessage};
use super::LanguageServerProtocol;

use lsp_server::ResponseError;
use lsp_types::{DidChangeTextDocumentParams, Url};

pub struct LangServerHandler {
    actors: RwLock<HashMap<Url, RwLock<Box<dyn LanguageServerProtocol + Send + Sync>>>>,
    config: LSPConfig,
}

impl LangServerHandler {
    pub fn new(config: LSPConfig) -> Self {
        Self {
            actors: RwLock::new(HashMap::new()),
            config,
        }
    }

    pub async fn open_file(
        &self,
        lang_id: &str,
        url: Url,
        text: &str,
    ) -> Result<(), ResponseError> {
        let actor = match lang_id.to_lowercase().as_str() {
            "asm" => AssemblyLanguageServerProtocol::new(&text, url.clone(), self.config.clone()),
            "assembly" => {
                AssemblyLanguageServerProtocol::new(&text, url.clone(), self.config.clone())
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
        let change_params = msg.content_changes;
        if change_params.len() != 1 {
            panic!("Unexpected document changed parameters");
        }

        let text = &change_params.first().unwrap().text;
        self.actors
            .read()
            .await
            .get(&uri)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?
            .write()
            .await
            .update(text);

        Ok(())
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
        request: SemanticTokensMessage,
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
}
