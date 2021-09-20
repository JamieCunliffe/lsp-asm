use std::path::Path;

use std::convert::Infallible;

use cucumber_rust::gherkin::Step;
use cucumber_rust::{async_trait, given, then, when, World, WorldInit};

use base::Architecture;
use lsp_asm::handler::types::{DocumentRangeMessage, FindReferencesMessage, LocationMessage};
use lsp_types::{
    DidChangeTextDocumentParams, MarkupContent, SemanticTokens, SemanticTokensResult,
    TextDocumentContentChangeEvent, Url, VersionedTextDocumentIdentifier,
};

use pretty_assertions::assert_eq;
use serde_json::Value;
use util::{get_doc_position, parse_config};

use lsp_asm::handler::handlers::LangServerHandler;

use crate::util::sort_completions;

mod util;

#[tokio::main]
async fn main() {
    let runner = LSPWorld::init(&["./features"]);
    runner.cli().run_and_exit().await
}

struct LastResponse {
    pub cmd: String,
    pub file: Url,
    pub resp: Value,
}
impl LastResponse {
    pub fn new(cmd: String, file: Url, resp: Value) -> Self {
        Self { cmd, file, resp }
    }
}
impl Default for LastResponse {
    fn default() -> Self {
        LastResponse::new(
            "".into(),
            Url::parse("file://").unwrap(),
            Default::default(),
        )
    }
}

#[derive(WorldInit)]
pub struct LSPWorld {
    pub handler: LangServerHandler,
    last_response: LastResponse,
}

#[async_trait(?Send)]
impl World for LSPWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Self::Error> {
        Ok(LSPWorld {
            handler: LangServerHandler::new(Default::default()),
            last_response: Default::default(),
        })
    }
}

#[given(regex = r#"I have the "(.*)" documentation"#)]
async fn check_doc(_state: &mut LSPWorld, arch: String) {
    let arch = Architecture::from(arch.as_str());
    let data = std::fs::read_to_string(format!("./features/known-defs/{}.json", &arch)).unwrap();
    let data = serde_json::from_str(data.as_str()).unwrap();
    documentation::poison_cache(&arch, data);
}

#[given("an lsp initialized with the following parameters")]
async fn init_lsp(state: &mut LSPWorld, step: &Step) {
    let config = parse_config(&step.table.as_ref().unwrap().rows);
    state.handler = LangServerHandler::new(config);
}

#[when(regex = r#"I open the temporary file "(.*)""#)]
async fn open_temp_file(state: &mut LSPWorld, step: &Step, name: String) {
    let data = step.docstring.as_ref().unwrap();
    let data = &data[1..data.len() - 1];
    let url = Url::parse(&format!("file://{}", name)).unwrap();

    state.handler.open_file("asm", url, &data, 0).await.unwrap();
}

#[when(regex = r#"I open the file "(.*)""#)]
async fn open_file(state: &mut LSPWorld, file: String) {
    let path = Path::new(&file);
    let data = std::fs::read_to_string(path).unwrap();
    let url = Url::parse(&format!("file://{}", file)).unwrap();

    state.handler.open_file("asm", url, &data, 0).await.unwrap();
}

#[when(
    regex = r#"I insert the following text in "(.*)" at position "(.*)" to bring it to version ([0-9]+)"#
)]
async fn insert_file(state: &mut LSPWorld, step: &Step, file: String, pos: String, version: i32) {
    let pos = get_doc_position(pos.as_str());
    let url = Url::parse(&format!("file://{}", file)).unwrap();
    let data = step.docstring.as_ref().unwrap();
    let data = &data[1..data.len() - 1];

    state
        .handler
        .update_file(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier::new(url, version),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(lsp_types::Range::new(pos.clone().into(), pos.into())),
                range_length: None,
                text: data.to_string(),
            }],
        })
        .await
        .unwrap();
}

#[when(
    regex = r#"I update the following text in "(.*)" at position "(.*)" to bring it to version ([0-9]+)"#
)]
async fn update_file(state: &mut LSPWorld, step: &Step, file: String, pos: String, version: i32) {
    let pos = util::make_range(&pos);
    let url = Url::parse(&format!("file://{}", file)).unwrap();
    let data = step.docstring.as_ref().unwrap();
    let data = &data[1..data.len() - 1];

    state
        .handler
        .update_file(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier::new(url, version),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: pos.into(),
                range_length: None,
                text: data.to_string(),
            }],
        })
        .await
        .unwrap();
}

#[when(regex = r#"I perform a full sync of the file "(.*)" to bring it to version ([0-9]+)"#)]
async fn full_sync_file(state: &mut LSPWorld, step: &Step, file: String, version: i32) {
    let url = Url::parse(&format!("file://{}", file)).unwrap();
    let data = step.docstring.as_ref().unwrap();
    let data = &data[1..data.len() - 1];

    state
        .handler
        .update_file(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier::new(url, version),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: data.to_string(),
            }],
        })
        .await
        .unwrap();
}

#[when(regex = r#"I run "(.*)" on the file "(.*)" at position "(.*)"(.*)"#)]
async fn run_command(
    state: &mut LSPWorld,
    cmd: String,
    file: String,
    pos: String,
    additional: String,
) {
    let (doc_pos, range) = if pos.contains('-') {
        let r = util::make_range(&pos);
        (r.start.into(), Some(r.into()))
    } else {
        (get_doc_position(&pos), None)
    };
    let url = Url::parse(&format!("file://{}", file)).unwrap();
    let additional = additional.trim();
    let location = LocationMessage {
        url: url.clone(),
        position: doc_pos,
    };
    let handler = &state.handler;
    let resp = match cmd.as_str() {
        "goto definition" => util::make_result(&handler.goto_definition(location).await),
        "find references" => {
            let req = FindReferencesMessage {
                location,
                include_decl: additional == "including decl",
            };
            util::make_result(&handler.find_references(req).await)
        }
        "document highlight" => util::make_result(&handler.document_highlight(location).await),
        "document hover" => util::make_result(&handler.hover(location).await),
        "semantic tokens" => {
            let req = DocumentRangeMessage {
                url: location.url,
                range,
            };
            util::make_result(&handler.get_semantic_tokens(req).await)
        }
        "document symbols" => util::make_result(&handler.document_symbols(location.url).await),
        "codelens" => util::make_result(&handler.code_lens(location.url).await),
        "syntax tree" => util::make_result(&handler.syntax_tree(location.url).await),
        "completion" => util::make_result(
            &handler
                .completion(location)
                .await
                .map(|i| sort_completions(i)),
        ),
        _ => "".into(),
    };

    state.last_response = LastResponse::new(cmd, url, resp);
}

#[then("I expect the following response")]
fn expect_response(state: &mut LSPWorld, step: &Step) {
    let actual = serde_json::to_value(&state.last_response.resp).unwrap();
    let cmd = &state.last_response.cmd;
    let file = &state.last_response.file;

    let expected = if let Some(expected) = step.docstring.as_ref() {
        match cmd.as_ref() {
            "document hover" => serde_json::to_value(lsp_types::Hover {
                contents: lsp_types::HoverContents::Markup(MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: expected[1..expected.len() - 1].to_string(),
                }),
                range: None,
            }),
            "syntax tree" => serde_json::to_value(expected.trim_start()),
            _ => serde_json::from_str(expected),
        }
    } else if let Some(expected) = step.table.as_ref() {
        let rows = &expected.rows;
        match cmd.as_ref() {
            "goto definition" => serde_json::to_value(lsp_types::GotoDefinitionResponse::Array(
                util::make_lsp_doc_location(file, rows),
            )),
            "find references" => serde_json::to_value(lsp_types::GotoDefinitionResponse::Array(
                util::make_lsp_doc_location(file, rows),
            )),
            "document highlight" => serde_json::to_value(util::make_doc_highlight(rows)),
            "semantic tokens" => {
                serde_json::to_value(SemanticTokensResult::Tokens(SemanticTokens {
                    data: util::make_semantic(rows),
                    ..Default::default()
                }))
            }
            "document symbols" => serde_json::to_value(util::make_doc_symbol(rows)),
            "codelens" => serde_json::to_value(util::make_codelens(rows)),
            "completion" => serde_json::to_value(sort_completions(util::make_completion(rows))),
            _ => panic!("Unknown cmd: {}", cmd),
        }
    } else {
        panic!("No response found");
    }
    .unwrap();

    assert_eq!(actual, expected);
}
