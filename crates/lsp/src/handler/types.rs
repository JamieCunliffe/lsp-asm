use crate::types::{DocumentLocation, DocumentPosition, DocumentRange};
use lsp_types::{TextDocumentPositionParams, Url};

impl From<lsp_types::TextDocumentPositionParams> for DocumentPosition {
    fn from(pos: lsp_types::TextDocumentPositionParams) -> Self {
        Self {
            line: pos.position.line,
            column: pos.position.character,
        }
    }
}

impl From<DocumentPosition> for lsp_types::Position {
    fn from(val: DocumentPosition) -> Self {
        lsp_types::Position::new(val.line, val.column)
    }
}

impl From<lsp_types::Position> for DocumentPosition {
    fn from(pos: lsp_types::Position) -> Self {
        DocumentPosition {
            line: pos.line,
            column: pos.character,
        }
    }
}

impl From<lsp_types::Range> for DocumentRange {
    fn from(r: lsp_types::Range) -> Self {
        Self {
            start: r.start.into(),
            end: r.end.into(),
        }
    }
}
impl From<DocumentRange> for lsp_types::Range {
    fn from(val: DocumentRange) -> Self {
        lsp_types::Range::new(val.start.into(), val.end.into())
    }
}

impl From<DocumentLocation> for lsp_types::Location {
    fn from(val: DocumentLocation) -> Self {
        Self::new(val.uri, val.range.into())
    }
}

pub struct LocationMessage {
    pub url: Url,
    pub position: DocumentPosition,
}
impl From<TextDocumentPositionParams> for LocationMessage {
    fn from(p: TextDocumentPositionParams) -> Self {
        Self {
            url: p.text_document.uri,
            position: p.position.into(),
        }
    }
}
impl From<lsp_types::GotoDefinitionParams> for LocationMessage {
    fn from(p: lsp_types::GotoDefinitionParams) -> Self {
        p.text_document_position_params.into()
    }
}
impl From<lsp_types::HoverParams> for LocationMessage {
    fn from(p: lsp_types::HoverParams) -> Self {
        p.text_document_position_params.into()
    }
}
impl From<lsp_types::DocumentHighlightParams> for LocationMessage {
    fn from(p: lsp_types::DocumentHighlightParams) -> Self {
        p.text_document_position_params.into()
    }
}

pub struct FindReferencesMessage {
    pub location: LocationMessage,
    pub include_decl: bool,
}
impl From<lsp_types::ReferenceParams> for FindReferencesMessage {
    fn from(p: lsp_types::ReferenceParams) -> Self {
        Self {
            location: p.text_document_position.into(),
            include_decl: p.context.include_declaration,
        }
    }
}

pub struct SemanticTokensMessage {
    pub url: Url,
    pub range: Option<DocumentRange>,
}
impl SemanticTokensMessage {
    pub fn new(url: Url, range: Option<DocumentRange>) -> Self {
        Self { url, range }
    }
}