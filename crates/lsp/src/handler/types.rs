use crate::types::{CompletionItem, DocumentLocation, DocumentPosition, DocumentRange};
use lsp_types::{
    CompletionItemKind, Documentation, MarkupContent, TextDocumentPositionParams, Url,
};

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

#[derive(Debug)]
pub struct DocumentRangeMessage {
    pub url: Url,
    pub range: Option<DocumentRange>,
}
impl DocumentRangeMessage {
    pub fn new(url: Url, range: Option<DocumentRange>) -> Self {
        Self { url, range }
    }
}
impl From<super::ext::RunAnalysisParams> for DocumentRangeMessage {
    fn from(p: super::ext::RunAnalysisParams) -> Self {
        Self {
            url: p.text_document.uri,
            range: p.range.map(|r| r.into()),
        }
    }
}

pub struct DocumentChange {
    pub(crate) text: String,
    pub(crate) range: Option<DocumentRange>,
}

impl From<CompletionItem> for lsp_types::CompletionItem {
    fn from(item: CompletionItem) -> Self {
        Self {
            label: item.text,
            label_details: None,
            kind: Some(match item.kind {
                crate::types::CompletionKind::Label => CompletionItemKind::CONSTANT,
                crate::types::CompletionKind::Register => CompletionItemKind::VARIABLE,
                crate::types::CompletionKind::Mnemonic => CompletionItemKind::FUNCTION,
                crate::types::CompletionKind::Text => CompletionItemKind::TEXT,
            }),
            detail: Some(item.details),
            documentation: item.documentation.map(|d| {
                Documentation::MarkupContent(MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: d,
                })
            }),
            deprecated: None,
            preselect: None,
            sort_text: None,
            filter_text: None,
            insert_text: None,
            insert_text_format: None,
            insert_text_mode: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: None,
            data: None,
            tags: None,
        }
    }
}
