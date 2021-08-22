use lsp_types::Url;

pub type LineNumber = u32;
pub type ColumnNumber = u32;

#[derive(Debug, PartialEq, Clone)]
pub struct DocumentPosition {
    /// 0 based line index
    pub line: LineNumber,
    pub column: ColumnNumber,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentRange {
    pub start: DocumentPosition,
    pub end: DocumentPosition,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DocumentLocation {
    pub uri: Url,
    pub range: DocumentRange,
}
