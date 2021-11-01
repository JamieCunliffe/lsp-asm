#[macro_use]
extern crate log;

use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response, ResponseError};

use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
};
use lsp_types::request::{
    CodeLensRequest, Completion, DocumentHighlightRequest, DocumentSymbolRequest, GotoDefinition,
    HoverRequest, References, SemanticTokensFullRequest, SemanticTokensRangeRequest,
    SignatureHelpRequest,
};
use lsp_types::{
    CodeLensOptions, CompletionOptions, CompletionOptionsCompletionItem, HoverProviderCapability,
    OneOf, PublishDiagnosticsParams, SemanticTokensFullOptions, SemanticTokensLegend,
    SemanticTokensOptions, SemanticTokensServerCapabilities, ServerCapabilities,
    SignatureHelpOptions, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    WorkDoneProgressOptions,
};

use serde_json::Value;

use lsp_asm::handler::handlers::LangServerHandler;
use lsp_asm::handler::types::DocumentRangeMessage;
use tokio::runtime::Builder;

pub(crate) type LangServerResult = Result<Value, ResponseError>;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    pretty_env_logger::init_timed();
    debug!("Starting lsp-asm");

    let (connection, io_threads) = Connection::stdio();

    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::Incremental,
        )),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        document_highlight_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec![String::from(" ")]),
            retrigger_characters: Some(vec![String::from(",")]),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        }),
        code_lens_provider: Some(CodeLensOptions {
            resolve_provider: None,
        }),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: None,
                },
                legend: SemanticTokensLegend {
                    token_types: lsp_asm::handler::semantic::TOKEN_TYPES.to_vec(),
                    token_modifiers: vec![],
                },

                full: Some(SemanticTokensFullOptions::Bool(true)),
                range: Some(true),
            },
        )),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![String::from(" "), String::from(",")]),
            all_commit_characters: None,
            work_done_progress_options: Default::default(),
            completion_item: Some(CompletionOptionsCompletionItem {
                label_details_support: Some(true),
            }),
        }),
        ..Default::default()
    };

    let server_capabilities = serde_json::to_value(&capabilities).unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;
    let initialization_params = serde_json::from_value(initialization_params)?;
    lsp_loop(connection, initialization_params)?;
    io_threads.join()?;

    debug!("Shutting server down");
    Ok(())
}

fn lsp_loop(
    connection: Connection,
    params: lsp_types::InitializeParams,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    debug!("Starting lsp loop");
    info!("Initialization params: {:#?}", params);
    let root = params
        .root_uri
        .and_then(|uri| uri.to_file_path().ok())
        .and_then(|path| path.to_str().map(|p| p.to_string()))
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.as_os_str().to_str().map(|p| p.to_string()))
        })
        .unwrap_or_default();

    let params = params
        .initialization_options
        .map(|opts| match serde_json::from_value(opts) {
            Ok(c) => Some(c),
            Err(e) => {
                error!("Failed to parse config due to error: {:#?}", e);
                None
            }
        })
        .flatten()
        .unwrap_or_default();

    info!("Config: {:#?}", params);
    let handler = Arc::new(LangServerHandler::new(params, root));
    let connection = Arc::new(connection);
    let rt = Builder::new_multi_thread()
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
            format!("lsp-asm-worker-{}", id)
        })
        .build()?;

    debug!("runtime: {:#?}", rt);

    rt.block_on(async move {
        for msg in &connection.receiver {
            let connection = connection.clone();
            let handler = handler.clone();
            match msg {
                Message::Request(req)
                    if connection.clone().handle_shutdown(&req).unwrap_or(false) =>
                {
                    return
                }

                m => tokio::spawn(async move {
                    process_message(connection.clone(), handler.clone(), m).await
                }),
            };
        }
    });

    Ok(())
}

async fn process_message(
    connection: Arc<Connection>,
    handler: Arc<LangServerHandler>,
    msg: Message,
) {
    match msg {
        Message::Request(request) => {
            let req_id = request.id.clone();
            info!("Handling request: {:#?}, id: {}", request.method, &req_id);
            let result = match request.method.as_str() {
                "textDocument/completion" => {
                    let (_, data) = get_message::<Completion>(request).unwrap();
                    make_result(handler.completion(data.text_document_position.into()).await)
                }
                "textDocument/definition" => {
                    let (_, data) = get_message::<GotoDefinition>(request).unwrap();
                    make_result(handler.goto_definition(data.into()).await)
                }
                "textDocument/references" => {
                    let (_, data) = get_message::<References>(request).unwrap();
                    make_result(handler.find_references(data.into()).await)
                }
                "textDocument/hover" => {
                    let (_, data) = get_message::<HoverRequest>(request).unwrap();
                    make_result(handler.hover(data.into()).await)
                }
                "textDocument/documentSymbol" => {
                    let (_, data) = get_message::<DocumentSymbolRequest>(request).unwrap();
                    make_result(handler.document_symbols(data.text_document.uri).await)
                }
                "textDocument/documentHighlight" => {
                    let (_, data) = get_message::<DocumentHighlightRequest>(request).unwrap();
                    make_result(handler.document_highlight(data.into()).await)
                }
                "textDocument/semanticTokens/full" => {
                    let (_, data) = get_message::<SemanticTokensFullRequest>(request).unwrap();
                    let msg = DocumentRangeMessage::new(data.text_document.uri, None);
                    make_result(handler.get_semantic_tokens(msg).await)
                }
                "textDocument/semanticTokens/range" => {
                    let (_, data) = get_message::<SemanticTokensRangeRequest>(request).unwrap();
                    let msg =
                        DocumentRangeMessage::new(data.text_document.uri, Some(data.range.into()));
                    make_result(handler.get_semantic_tokens(msg).await)
                }
                "textDocument/codeLens" => {
                    let (_, data) = get_message::<CodeLensRequest>(request).unwrap();
                    make_result(handler.code_lens(data.text_document.uri).await)
                }
                "textDocument/signatureHelp" => {
                    let (_, data) = get_message::<SignatureHelpRequest>(request).unwrap();
                    make_result(
                        handler
                            .signature_help(&data.text_document_position_params.into())
                            .await,
                    )
                }
                "asm/syntaxTree" => {
                    let (_, data) =
                        get_message::<lsp_asm::handler::ext::SyntaxTree>(request).unwrap();
                    make_result(handler.syntax_tree(data.text_document.uri).await)
                }
                "asm/runAnalysis" => {
                    let (_, data) =
                        get_message::<lsp_asm::handler::ext::RunAnalysis>(request).unwrap();
                    make_result(handler.analysis(data.into()).await)
                }
                _ => panic!("Unknown method: {:?}", request.method),
            };
            info!("Responding to request: {}", &req_id);
            let response = make_response(req_id, result);
            let res = connection.sender.send(Message::Response(response));
            match res {
                Ok(_) => (),
                Err(e) => error!("Failed to respond to request due to error: {:#?}", e),
            };
        }

        Message::Notification(notification) => {
            info!("Handling notification: {:#?}", notification.method);
            let _ = match notification.method.as_str() {
                "textDocument/didOpen" => {
                    let data = get_notification::<DidOpenTextDocument>(notification).unwrap();
                    let data = data.text_document;
                    let _ = handler
                        .open_file(
                            &data.language_id,
                            data.uri.clone(),
                            &data.text,
                            data.version as _,
                        )
                        .await;
                    handle_diagnostics(connection.clone(), handler.clone(), data.uri);
                    Ok(())
                }
                "textDocument/didChange" => {
                    let data = get_notification::<DidChangeTextDocument>(notification).unwrap();

                    if let Err(e) = handler.update_file(data).await {
                        if let Some(params) = e.data {
                            let _ = connection.sender.send(Message::Notification(Notification {
                                method: String::from("textDocument/resync"),
                                params,
                            }));
                        }
                    }

                    Ok(())
                }
                "textDocument/didSave" => {
                    let data = get_notification::<DidSaveTextDocument>(notification).unwrap();
                    handle_diagnostics(connection.clone(), handler.clone(), data.text_document.uri);
                    Ok(())
                }
                "textDocument/didClose" => {
                    let data = get_notification::<DidCloseTextDocument>(notification).unwrap();
                    handler.close_file(data.text_document.uri).await
                }
                "$/cancelRequest" => {
                    debug!("Received cancel request - Ignoring");
                    Ok(())
                }
                _ => {
                    error!("Unknown notification: {:?}", notification.method);
                    Ok(())
                }
            };
        }
        _ => (),
    }
}

fn handle_diagnostics(connection: Arc<Connection>, handler: Arc<LangServerHandler>, uri: Url) {
    if !handler.config().diagnostics.enabled {
        return;
    }
    info!("Handling diagnostics for file: {}", uri);

    tokio::spawn(async move {
        let diagnostics = handler
            .get_diagnostics(&uri)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.into())
            .collect();

        let params = PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        };

        let diagnotics = Message::Notification(Notification {
            method: String::from("textDocument/publishDiagnostics"),
            params: serde_json::to_value(params).unwrap_or_default(),
        });

        let _ = connection.sender.send(diagnotics);
    });
}

fn make_result<T>(result: Result<T, ResponseError>) -> LangServerResult
where
    T: serde::Serialize,
{
    match result {
        Ok(result) => Ok(serde_json::to_value(&result).unwrap()),
        Err(result) => Err(result),
    }
}

fn make_response(id: RequestId, result: LangServerResult) -> Response {
    match result {
        Ok(result) => Response {
            id,
            result: Some(result),
            error: None,
        },
        Err(err) => Response {
            id,
            result: None,
            error: Some(err),
        },
    }
}

fn get_message<R>(req: Request) -> Option<(RequestId, R::Params)>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    let request = req.extract(R::METHOD);
    match request {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}

fn get_notification<R>(req: Notification) -> Option<R::Params>
where
    R: lsp_types::notification::Notification,
    R::Params: serde::de::DeserializeOwned,
{
    let request = req.extract(R::METHOD);
    match request {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}
