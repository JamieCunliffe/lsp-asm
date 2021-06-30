use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::asm::handler::AssemblyLanguageServerProtocol;
use crate::config::LSPConfig;

use super::error::{lsp_error_map, ErrorCode};
use super::types::{FindReferencesMessage, LocationMessage, SemanticTokensMessage};
use super::LanguageServerProtocol;

use lsp_server::ResponseError;
use lsp_types::{DidChangeTextDocumentParams, Url};

pub struct LangServerHandler {
    actors: HashMap<Url, Box<dyn LanguageServerProtocol>>,
    config: LSPConfig,
}

impl LangServerHandler {
    pub fn new(config: LSPConfig) -> Self {
        Self {
            actors: HashMap::new(),
            config,
        }
    }

    pub fn open_file(&mut self, lang_id: &str, url: Url, text: &str) {
        let actor = match lang_id.to_lowercase().as_str() {
            "asm" => AssemblyLanguageServerProtocol::new(&text, url.clone(), self.config.clone()),
            "assembly" => AssemblyLanguageServerProtocol::new(&text, url.clone(), self.config.clone()),
            _ => panic!("Unknown language: {}", lang_id),
        };

        self.actors.insert(url, Box::new(actor));
    }

    pub fn update_file(&mut self, msg: DidChangeTextDocumentParams) {
        let uri = msg.text_document.uri;
        let change_params = msg.content_changes;
        if change_params.len() != 1 {
            panic!("Unexpected document changed parameters");
        }

        let text = &change_params.first().unwrap().text;
        self.actors.get_mut(&uri).unwrap().update(text);
    }

    pub fn close_file(&mut self, url: Url) {
        if let Entry::Occupied(entry) = self.actors.entry(url) {
            entry.remove_entry();
        }
    }

    fn get_actor(&self, url: &Url) -> Result<&Box<dyn LanguageServerProtocol>, ResponseError> {
        Ok(self
            .actors
            .get(url)
            .ok_or_else(|| lsp_error_map(ErrorCode::FileNotFound))?)
    }

    pub fn goto_definition(
        &self,
        request: LocationMessage,
    ) -> Result<lsp_types::GotoDefinitionResponse, ResponseError> {
        let handler = self.get_actor(&request.url)?;

        handler.goto_definition(request.position)
    }

    pub fn find_references(
        &self,
        request: FindReferencesMessage,
    ) -> Result<Vec<lsp_types::Location>, ResponseError> {
        let handler = self.get_actor(&request.location.url)?;

        handler.find_references(request.location.position, request.include_decl)
    }

    pub fn hover(
        &self,
        request: LocationMessage,
    ) -> Result<Option<lsp_types::Hover>, ResponseError> {
        let handler = self.get_actor(&request.url)?;

        handler.hover(request.position)
    }

    pub fn document_highlight(
        &self,
        request: LocationMessage,
    ) -> Result<Vec<lsp_types::DocumentHighlight>, ResponseError> {
        let handler = self.get_actor(&request.url)?;

        handler.document_highlight(request.position)
    }

    pub fn get_semantic_tokens(
        &self,
        request: SemanticTokensMessage,
    ) -> Result<lsp_types::SemanticTokensResult, ResponseError> {
        let handler = self.get_actor(&request.url)?;

        handler.get_semantic_tokens(request.range.map(|r| r.into()))
    }

    pub fn document_symbols(
        &self,
        url: Url,
    ) -> Result<lsp_types::DocumentSymbolResponse, ResponseError> {
        let handler = self.get_actor(&url)?;

        handler.document_symbols()
    }
}
