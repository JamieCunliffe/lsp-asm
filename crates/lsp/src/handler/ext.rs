use lsp_types::request::Request;
use lsp_types::Range;
use lsp_types::TextDocumentIdentifier;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxTreeParams {
    pub text_document: TextDocumentIdentifier,
}

pub enum SyntaxTree {}

impl Request for SyntaxTree {
    type Params = SyntaxTreeParams;
    type Result = String;
    const METHOD: &'static str = "asm/syntaxTree";
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunAnalysisParams {
    pub text_document: TextDocumentIdentifier,
    pub range: Option<Range>,
}

pub enum RunAnalysis {}

impl Request for RunAnalysis {
    type Params = RunAnalysisParams;
    type Result = String;
    const METHOD: &'static str = "asm/runAnalysis";
}
