use lsp_server::ResponseError;
use lsp_types::{
    CompletionList, DocumentHighlight, DocumentSymbolResponse, GotoDefinitionResponse, Hover,
    Location, Range, SemanticTokensResult, SignatureHelp,
};

use crate::types::{DocumentPosition, DocumentRange};

use self::types::DocumentChange;

pub mod error;
pub mod ext;
pub mod handlers;
pub mod semantic;
pub mod types;

pub trait LanguageServerProtocol {
    fn update(&mut self, version: u32, changes: Vec<DocumentChange>) -> bool;

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

    fn code_lens(&self) -> Result<Option<Vec<lsp_types::CodeLens>>, ResponseError>;

    fn completion(&self, location: DocumentPosition) -> Result<CompletionList, ResponseError>;

    fn signature_help(
        &self,
        position: &DocumentPosition,
    ) -> Result<Option<SignatureHelp>, ResponseError>;

    fn format(
        &self,
        workspace_root: &str,
    ) -> Result<Option<Vec<lsp_types::TextEdit>>, ResponseError>;

    fn syntax_tree(&self) -> Result<String, ResponseError>;

    fn analysis(&self, range: Option<DocumentRange>) -> Result<String, ResponseError>;
}
