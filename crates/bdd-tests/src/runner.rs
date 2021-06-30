use std::path::Path;

use std::convert::Infallible;

use cucumber_rust::gherkin::Step;
use cucumber_rust::{World, WorldInit, async_trait, given, then, when};

use lsp_asm::handler::types::{FindReferencesMessage, LocationMessage, SemanticTokensMessage};
use lsp_types::{MarkupContent, SemanticTokens, SemanticTokensResult, Url};
use pretty_assertions::assert_eq;
use serde_json::Value;
use util::{get_doc_position, parse_config};

use lsp_asm::handler::handlers::LangServerHandler;

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

#[given("an lsp initialized with the following parameters")]
async fn init_lsp(state: &mut LSPWorld, step: &Step) {
    let config = parse_config(&step.table.as_ref().unwrap().rows);
    state.handler = LangServerHandler::new(config);
}

#[when(regex = r#"I open the file "(.*)""#)]
async fn open_file(state: &mut LSPWorld, file: String) {
    let path = Path::new(&file);
    let data = std::fs::read_to_string(path).unwrap();
    let url = Url::parse(&format!("file://{}", file)).unwrap();

    state.handler.open_file("asm", url, &data);
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
        "goto definition" => util::make_result(&handler.goto_definition(location)),
        "find references" => {
            let req = FindReferencesMessage {
                location,
                include_decl: additional == "including decl",
            };
            util::make_result(&handler.find_references(req))
        }
        "document highlight" => util::make_result(&handler.document_highlight(location)),
        "document hover" => util::make_result(&handler.hover(location)),
        "semantic tokens" => {
            let req = SemanticTokensMessage {
                url: location.url,
                range,
            };
            util::make_result(&handler.get_semantic_tokens(req))
        }
        "document symbols" => util::make_result(&handler.document_symbols(location.url)),
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
                    value: expected.to_string(),
                }),
                range: None,
            }),
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
            _ => panic!("Unknown cmd: {}", cmd),
        }
    } else {
        panic!("No response found");
    }
    .unwrap();

    assert_eq!(actual, expected);
}
