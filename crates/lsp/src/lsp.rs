use crate::handler::context::Context;
use crate::handler::types::DocumentRangeMessage;
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response, ResponseError};
use lsp_types::notification::{
    Cancel, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
};
use lsp_types::request::{
    CodeLensRequest, Completion, DocumentHighlightRequest, DocumentSymbolRequest, Formatting,
    GotoDefinition, HoverRequest, References, Rename, SemanticTokensFullRequest,
    SemanticTokensRangeRequest, SignatureHelpRequest,
};
use lsp_types::{PublishDiagnosticsParams, Url};
use rayon::ThreadPoolBuilder;
use serde_json::Value;
use std::error::Error;
use std::sync::Arc;

pub(crate) type LangServerResult = Result<Value, ResponseError>;

pub fn lsp_loop(
    connection: Arc<Connection>,
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
        .and_then(|opts| match serde_json::from_value(opts) {
            Ok(c) => Some(c),
            Err(e) => {
                error!("Failed to parse config due to error: {:#?}", e);
                None
            }
        })
        .unwrap_or_default();

    info!("Config: {:#?}", params);
    let context = Arc::new(Context::new(params, root));
    let thread_pool = ThreadPoolBuilder::new()
        .thread_name(|id| format!("lsp-asm-worker-{id}"))
        .panic_handler(|e| error!("Panic: {:#?}", e))
        .build()?;

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) if connection.clone().handle_shutdown(&req).unwrap_or(false) => {
                drop(thread_pool);
                return Ok(());
            }
            m => {
                let connection = connection.clone();
                let context = context.clone();
                thread_pool.spawn(move || process_message(connection.clone(), context.clone(), m));
            }
        };
    }

    Ok(())
}

fn process_message(connection: Arc<Connection>, context: Arc<Context>, msg: Message) {
    use crate::handler::handlers;

    match msg {
        Message::Request(request) => {
            let req_id = request.id.clone();
            info!("Handling request: {:#?}, id: {}", request, &req_id);
            let result = match request.method.as_str() {
                "textDocument/completion" => {
                    let (_, data) = get_message::<Completion>(request).unwrap();
                    make_result(handlers::completion(
                        context,
                        data.text_document_position.into(),
                    ))
                }
                "textDocument/definition" => {
                    let (_, data) = get_message::<GotoDefinition>(request).unwrap();
                    make_result(handlers::goto_definition(context, data.into()))
                }
                "textDocument/references" => {
                    let (_, data) = get_message::<References>(request).unwrap();
                    make_result(handlers::find_references(context, data.into()))
                }
                "textDocument/hover" => {
                    let (_, data) = get_message::<HoverRequest>(request).unwrap();
                    make_result(handlers::hover(context, data.into()))
                }
                "textDocument/documentSymbol" => {
                    let (_, data) = get_message::<DocumentSymbolRequest>(request).unwrap();
                    make_result(handlers::document_symbols(context, data.text_document.uri))
                }
                "textDocument/documentHighlight" => {
                    let (_, data) = get_message::<DocumentHighlightRequest>(request).unwrap();
                    make_result(handlers::document_highlight(context, data.into()))
                }
                "textDocument/semanticTokens/full" => {
                    let (_, data) = get_message::<SemanticTokensFullRequest>(request).unwrap();
                    let msg = DocumentRangeMessage::new(data.text_document.uri, None);
                    make_result(handlers::get_semantic_tokens(context, msg))
                }
                "textDocument/semanticTokens/range" => {
                    let (_, data) = get_message::<SemanticTokensRangeRequest>(request).unwrap();
                    let msg =
                        DocumentRangeMessage::new(data.text_document.uri, Some(data.range.into()));
                    make_result(handlers::get_semantic_tokens(context, msg))
                }
                "textDocument/codeLens" => {
                    let (_, data) = get_message::<CodeLensRequest>(request).unwrap();
                    make_result(handlers::code_lens(context, data.text_document.uri))
                }
                "textDocument/signatureHelp" => {
                    let (_, data) = get_message::<SignatureHelpRequest>(request).unwrap();
                    make_result(handlers::signature_help(
                        context,
                        &data.text_document_position_params.into(),
                    ))
                }
                "textDocument/formatting" => {
                    let (_, data) = get_message::<Formatting>(request).unwrap();
                    make_result(handlers::format(context, data.text_document.uri))
                }
                "textDocument/rename" => {
                    let (_, data) = get_message::<Rename>(request).unwrap();
                    make_result(handlers::rename(context, data.into()))
                }
                "asm/syntaxTree" => {
                    let (_, data) =
                        get_message::<crate::handler::ext::SyntaxTree>(request).unwrap();
                    make_result(handlers::syntax_tree(context, data.text_document.uri))
                }
                "asm/runAnalysis" => {
                    let (_, data) =
                        get_message::<crate::handler::ext::RunAnalysis>(request).unwrap();
                    make_result(handlers::analysis(context, data.into()))
                }
                "diag/fileStats" => {
                    let (_, data) = get_message::<crate::handler::ext::FileStats>(request).unwrap();
                    make_result(handlers::file_stats(context, data))
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
            info!("Handling notification: {}", notification.method);

            let _ = match notification.method.as_str() {
                "textDocument/didOpen" => {
                    let data = get_notification::<DidOpenTextDocument>(notification).unwrap();
                    let data = data.text_document;
                    let _ = handlers::open_file(
                        context.clone(),
                        &data.language_id,
                        data.uri.clone(),
                        &data.text,
                        data.version as _,
                    );
                    handle_diagnostics(connection, context, data.uri);
                    Ok(())
                }
                "textDocument/didChange" => {
                    let data = get_notification::<DidChangeTextDocument>(notification).unwrap();

                    if let Err(e) = handlers::update_file(context, data) {
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
                    handle_diagnostics(connection, context, data.text_document.uri);
                    Ok(())
                }
                "textDocument/didClose" => {
                    let data = get_notification::<DidCloseTextDocument>(notification).unwrap();
                    handlers::close_file(context, data.text_document.uri)
                }
                "$/cancelRequest" => {
                    let data = get_notification::<Cancel>(notification).unwrap();
                    info!("Received cancel request for: {:#?}", data.id);
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

fn handle_diagnostics(connection: Arc<Connection>, context: Arc<Context>, uri: Url) {
    use crate::handler::handlers;

    if !context.config().diagnostics.enabled {
        return;
    }
    info!("Handling diagnostics for file: {}", uri);

    let diagnostics = handlers::get_diagnostics(context, &uri)
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
