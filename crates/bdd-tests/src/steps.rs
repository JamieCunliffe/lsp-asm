use std::path::Path;

use base::Architecture;
use cucumber::gherkin::Step;
use cucumber::{given, then, when};
use lsp_asm::handler::ext::{SyntaxTree, SyntaxTreeParams};
use lsp_types::notification::DidChangeTextDocument;
use lsp_types::request::{
    CodeActionRequest, CodeLensRequest, Completion, DocumentHighlightRequest,
    DocumentSymbolRequest, GotoDefinition, HoverRequest, References, Rename,
    SemanticTokensRangeRequest, SignatureHelpRequest,
};
use lsp_types::{
    CodeActionParams, CodeLensParams, CompletionParams, DidChangeTextDocumentParams,
    DocumentHighlightParams, DocumentSymbolParams, GotoDefinitionParams, HoverParams,
    MarkupContent, ReferenceParams, RenameParams, SemanticTokens, SemanticTokensRangeParams,
    SemanticTokensResult, SignatureHelpParams, TextDocumentContentChangeEvent,
    TextDocumentPositionParams, VersionedTextDocumentIdentifier,
};
use pretty_assertions::assert_eq;

use crate::command::LSPCommand;
use crate::file::FileUrl;
use crate::position::PositionString;
use crate::util::parse_config;
use crate::{util, LSPWorld};

#[given(regex = r#"I have the "(.*)" documentation"#)]
async fn check_doc(_state: &mut LSPWorld, arch: String) {
    let arch = Architecture::from(arch.as_str());
    let data = std::fs::read_to_string(format!("./features/known-defs/{arch}.json")).unwrap();
    let data = serde_json::from_str(data.as_str()).unwrap();
    documentation::poison_cache(&arch, data);
}

#[given("an initialized lsp")]
async fn init_lsp_default(state: &mut LSPWorld) {
    state.lsp.init(Default::default(), None);
}

#[given("an lsp initialized with the following parameters")]
async fn init_lsp_with_config(state: &mut LSPWorld, step: &Step) {
    let config = parse_config(&step.table.as_ref().unwrap().rows);
    state.lsp.init(config, None);
}

#[given(regex = r#"an lsp initialized in "(.*)" with the following parameters"#)]
async fn init_lsp_with_location(state: &mut LSPWorld, step: &Step, root: String) {
    let root = std::path::Path::new(&root)
        .canonicalize()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();

    let config = parse_config(&step.table.as_ref().unwrap().rows);
    state.lsp.init(config, Some(root));
}

#[when(regex = r#"I open the temporary file "(.*)""#)]
async fn open_temp_file(state: &mut LSPWorld, step: &Step, url: FileUrl) {
    let data = step.docstring.as_ref().unwrap();
    let data = &data[1..data.len() - 1];

    state.lsp.open_file(url.into(), data);
}

#[when(regex = r#"I open the file "(.*)""#)]
async fn open_file(state: &mut LSPWorld, file: String) {
    let path = Path::new(&file);
    let data = std::fs::read_to_string(path).unwrap();
    let url = util::file_to_uri(&file);

    state.lsp.open_file(url, &data);
}

#[when(regex = r#"I close the file "(.*)""#)]
async fn close_file(state: &mut LSPWorld, file: String) {
    let url = util::file_to_uri(&file);

    state.lsp.close_file(url);
}

#[when(
    regex = r#"I insert the following text in "(.*)" at position "(.*)" to bring it to version ([0-9]+)"#
)]
async fn insert_file(
    state: &mut LSPWorld,
    step: &Step,
    url: FileUrl,
    pos: PositionString,
    version: i32,
) {
    let data = step.docstring.as_ref().unwrap();
    let data = &data[1..data.len() - 1];

    state
        .lsp
        .send_notification::<DidChangeTextDocument>(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier::new(url.clone().into(), version),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(pos.into()),
                range_length: None,
                text: data.to_string(),
            }],
        });

    state.lsp.wait_for_file_version(url.into(), version);
}

#[when(
    regex = r#"I update the following text in "(.*)" at position "(.*)" to bring it to version ([0-9]+)"#
)]
async fn update_file(
    state: &mut LSPWorld,
    step: &Step,
    url: FileUrl,
    pos: PositionString,
    version: i32,
) {
    let data = step.docstring.as_ref().unwrap();
    let data = &data[1..data.len() - 1];

    state
        .lsp
        .send_notification::<DidChangeTextDocument>(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier::new(url.clone().into(), version),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(pos.into()),
                range_length: None,
                text: data.to_string(),
            }],
        });

    state.lsp.wait_for_file_version(url.into(), version);
}

#[when(regex = r#"I perform a full sync of the file "(.*)" to bring it to version ([0-9]+)"#)]
async fn full_sync_file(state: &mut LSPWorld, step: &Step, url: FileUrl, version: i32) {
    let data = step.docstring.as_ref().unwrap();
    let data = &data[1..data.len() - 1];

    state
        .lsp
        .send_notification::<DidChangeTextDocument>(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier::new(url.clone().into(), version),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: data.to_string(),
            }],
        });

    state.lsp.wait_for_file_version(url.into(), version);
}

#[when(regex = r#"I run "(.*)" on the file "(.*)" at position "(.*)"(.*)"#)]
async fn run_command(
    state: &mut LSPWorld,
    cmd: LSPCommand,
    uri: FileUrl,
    pos: PositionString,
    additional: String,
) {
    let additional = additional.trim();

    let doc_position_params = || TextDocumentPositionParams {
        text_document: uri.to_text_document(),
        position: pos.clone().into(),
    };

    state.last_id = match cmd {
        LSPCommand::GotoDefinition => {
            state
                .lsp
                .send_request::<GotoDefinition>(GotoDefinitionParams {
                    text_document_position_params: doc_position_params(),
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                })
        }
        LSPCommand::FindReferences => state.lsp.send_request::<References>(ReferenceParams {
            text_document_position: doc_position_params(),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: lsp_types::ReferenceContext {
                include_declaration: additional == "including decl",
            },
        }),
        LSPCommand::DocumentHighlight => {
            state
                .lsp
                .send_request::<DocumentHighlightRequest>(DocumentHighlightParams {
                    text_document_position_params: doc_position_params(),
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                })
        }
        LSPCommand::DocumentHover => state.lsp.send_request::<HoverRequest>(HoverParams {
            text_document_position_params: doc_position_params(),
            work_done_progress_params: Default::default(),
        }),
        LSPCommand::SemanticTokens => {
            state
                .lsp
                .send_request::<SemanticTokensRangeRequest>(SemanticTokensRangeParams {
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                    text_document: uri.to_text_document(),
                    range: pos.into(),
                })
        }
        LSPCommand::DocumentSymbols => {
            state
                .lsp
                .send_request::<DocumentSymbolRequest>(DocumentSymbolParams {
                    text_document: uri.to_text_document(),
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                })
        }
        LSPCommand::Codelens => state.lsp.send_request::<CodeLensRequest>(CodeLensParams {
            text_document: uri.to_text_document(),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        }),
        LSPCommand::CodeAction => state
            .lsp
            .send_request::<CodeActionRequest>(CodeActionParams {
                text_document: uri.to_text_document(),
                range: pos.into(),
                context: Default::default(),
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            }),
        LSPCommand::SyntaxTree => state.lsp.send_request::<SyntaxTree>(SyntaxTreeParams {
            text_document: uri.to_text_document(),
        }),
        LSPCommand::Completion => state.lsp.send_request::<Completion>(CompletionParams {
            text_document_position: doc_position_params(),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: Default::default(),
        }),
        LSPCommand::SignatureHelp => {
            state
                .lsp
                .send_request::<SignatureHelpRequest>(SignatureHelpParams {
                    context: Default::default(),
                    text_document_position_params: doc_position_params(),
                    work_done_progress_params: Default::default(),
                })
        }
        LSPCommand::Rename => {
            let new_name = additional.trim_start_matches("with the new name: ");
            state.lsp.send_request::<Rename>(RenameParams {
                new_name: new_name.to_string(),
                work_done_progress_params: Default::default(),
                text_document_position: doc_position_params(),
            })
        }
        LSPCommand::NoCommand => panic!("Unknown command"),
    };

    state.last_cmd = cmd;
    state.last_file = uri;
}

#[then("I expect the following response")]
fn expect_response(state: &mut LSPWorld, step: &Step) {
    let actual = state.lsp.wait_for_response_for_id(state.last_id);
    let file = &state.last_file;
    let cmd = &state.last_cmd;

    let expected = if let Some(expected) = step.docstring.as_ref() {
        match cmd {
            LSPCommand::DocumentHover => serde_json::to_value(lsp_types::Hover {
                contents: lsp_types::HoverContents::Markup(MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: expected[1..expected.len() - 1].to_string(),
                }),
                range: None,
            }),
            LSPCommand::SyntaxTree => serde_json::to_value(expected.trim_start()),
            _ => serde_json::from_str(expected),
        }
    } else if let Some(expected) = step.table.as_ref() {
        let rows = &expected.rows;
        match cmd {
            LSPCommand::GotoDefinition => serde_json::to_value(
                lsp_types::GotoDefinitionResponse::Array(util::make_lsp_doc_location(file, rows)),
            ),
            LSPCommand::FindReferences => serde_json::to_value(
                lsp_types::GotoDefinitionResponse::Array(util::make_lsp_doc_location(file, rows)),
            ),
            LSPCommand::DocumentHighlight => serde_json::to_value(util::make_doc_highlight(rows)),
            LSPCommand::SemanticTokens => {
                serde_json::to_value(SemanticTokensResult::Tokens(SemanticTokens {
                    data: util::make_semantic(rows),
                    ..Default::default()
                }))
            }
            LSPCommand::DocumentSymbols => serde_json::to_value(util::make_doc_symbol(rows)),
            LSPCommand::Codelens => serde_json::to_value(util::make_codelens(rows)),
            LSPCommand::CodeAction => serde_json::to_value(util::make_codeaction(rows)),
            LSPCommand::Completion => serde_json::to_value(util::make_completion(rows)),
            LSPCommand::SignatureHelp => serde_json::to_value(util::make_signature_help(rows)),
            &LSPCommand::Rename => serde_json::to_value(util::make_workspace_edit(rows)),
            cmd => panic!("Unknown cmd: {cmd:#?}"),
        }
    } else {
        panic!("No response found");
    }
    .unwrap();

    assert_eq!(actual, expected);
}

#[then(regex = r#"I expect the error "(.*)""#)]
fn expect_lsp_error(state: &mut LSPWorld, error: String) {
    let actual = state.lsp.wait_for_error_for_id(state.last_id);
    assert_eq!(error, actual.message);
}

#[then(regex = r#"I expect the following errors for "(.*)""#)]
fn expect_errors(state: &mut LSPWorld, step: &Step, uri: FileUrl) {
    let rows = &step.table.as_ref().unwrap().rows;
    let expected_errors = util::get_errors(rows, uri.clone().into());

    let actual = state
        .lsp
        .diagnostics_for_file(&uri.into())
        .last()
        .unwrap()
        .clone();

    assert_eq!(actual, expected_errors);
}
