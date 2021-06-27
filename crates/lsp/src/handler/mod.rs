use lsp_server::ResponseError;
use lsp_types::{
    DocumentHighlight, DocumentSymbolResponse, GotoDefinitionResponse, Hover, Location, Range,
    SemanticTokensResult,
};

use crate::types::{DocumentPosition, LineNumber};

pub mod error;
pub mod handlers;
pub(crate) mod semantic;
pub mod types;

#[derive(Debug, PartialEq)]
pub struct LanguageServerProtocolConfig {
    pub(crate) visible_lines: LineNumber,
}

impl Default for LanguageServerProtocolConfig {
    fn default() -> Self {
        Self { visible_lines: 200 }
    }
}

pub trait LanguageServerProtocol {
    fn update(&mut self, data: &str);

    fn goto_definition(
        &self,
        position: DocumentPosition,
    ) -> Result<GotoDefinitionResponse, ResponseError>;

    fn find_references(
        &self,
        position: DocumentPosition,
        include_decl: bool,
    ) -> Result<Vec<Location>, ResponseError>;

    fn hover(&self, position: DocumentPosition) -> Result<Option<Hover>, ResponseError>;

    fn document_highlight(
        &self,
        position: DocumentPosition,
    ) -> Result<Vec<DocumentHighlight>, ResponseError>;

    fn get_semantic_tokens(
        &self,
        range: Option<Range>,
    ) -> Result<SemanticTokensResult, ResponseError>;

    fn document_symbols(&self) -> Result<DocumentSymbolResponse, ResponseError>;
}
