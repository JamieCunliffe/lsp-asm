use lsp_server::ResponseError;
use lsp_types::{
    DocumentHighlight, DocumentSymbolResponse, GotoDefinitionResponse, Hover, Location, Position,
    Range, SemanticTokensResult, TextDocumentPositionParams,
};

use crate::types::{DocumentPosition, DocumentRange, LineNumber};

pub mod handlers;
pub(crate) mod semantic;

#[derive(Debug, PartialEq)]
pub struct LanguageServerProtocolConfig {
    pub(crate) visible_lines: LineNumber,
}

impl Default for LanguageServerProtocolConfig {
    fn default() -> Self {
        Self { visible_lines: 200 }
    }
}

impl From<TextDocumentPositionParams> for DocumentPosition {
    fn from(pos: TextDocumentPositionParams) -> Self {
        Self {
            line: pos.position.line,
            column: pos.position.character,
        }
    }
}

impl From<DocumentPosition> for Position {
    fn from(val: DocumentPosition) -> Self {
        Position::new(val.line, val.column)
    }
}

impl From<Position> for DocumentPosition {
    fn from(pos: Position) -> Self {
        DocumentPosition {
            line: pos.line,
            column: pos.character,
        }
    }
}

impl From<DocumentRange> for Range {
    fn from(val: DocumentRange) -> Self {
        Range::new(val.start.into(), val.end.into())
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

#[cfg(test)]
mod test {
    use lsp_types::{Position, TextDocumentIdentifier, Url};

    use super::*;

    #[test]
    fn test_lsp_config_default() {
        let default_values = LanguageServerProtocolConfig { visible_lines: 200 };

        assert_eq!(default_values, LanguageServerProtocolConfig::default());
    }

    #[test]
    fn test_lsp_doc_pos_to_document_position() {
        let lsp = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: Url::parse("file://test").unwrap(),
            },
            position: Position {
                line: 54,
                character: 42,
            },
        };

        let position = DocumentPosition {
            line: 54,
            column: 42,
        };

        assert_eq!(DocumentPosition::from(lsp), position);
    }
}
