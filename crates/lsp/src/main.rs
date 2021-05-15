#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use std::error::Error;

use actix::Actor;

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response, ResponseError};

use lsp_types::notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument};
use lsp_types::request::{
    DocumentHighlightRequest, DocumentSymbolRequest, GotoDefinition, HoverRequest, References,
    SemanticTokensFullRequest, SemanticTokensRangeRequest,
};
use lsp_types::{
    HoverProviderCapability, OneOf, SemanticTokensFullOptions, SemanticTokensLegend,
    SemanticTokensOptions, SemanticTokensServerCapabilities, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
};

use serde_json::Value;

mod asm;
mod handler;
mod types;

use handler::handlers::{
    DocChangedNotification, DocClosedNotification, DocOpenNotification, DocumentHighlightMessage,
    DocumentSymbolMessage, FindReferencesMessage, GotoDefinitionMessage, HoverRequestMessage,
    LangServerHandler, LangServerResult, SemanticTokensMessage,
};

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    pretty_env_logger::init();
    debug!("Starting lsp-asm");

    let (connection, io_threads) = Connection::stdio();

    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::Full)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        document_highlight_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: None,
                },
                legend: SemanticTokensLegend {
                    token_types: handler::semantic::TOKEN_TYPES.to_vec(),
                    token_modifiers: vec![],
                },

                full: Some(SemanticTokensFullOptions::Bool(true)),
                range: Some(true),
            },
        )),
        ..ServerCapabilities::default()
    };

    let server_capabilities = serde_json::to_value(&capabilities).unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;

    lsp_loop(&connection, initialization_params).await?;
    io_threads.join()?;

    debug!("Shutting server down");
    Ok(())
}

async fn lsp_loop(
    connection: &Connection,
    _params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    debug!("Starting lsp loop");
    let handler = LangServerHandler::default().start();

    for msg in &connection.receiver {
        match msg {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    debug!("Handling shutdown");
                    return Ok(());
                }

                debug!("Handling request: {:#?}", request.method);
                let req_id = request.id.clone();
                let result = match request.method.as_str() {
                    "textDocument/definition" => {
                        let (_, data) = get_message::<GotoDefinition>(request).unwrap();
                        handler.send(GotoDefinitionMessage { data }).await?
                    }
                    "textDocument/references" => {
                        let (_, data) = get_message::<References>(request).unwrap();
                        handler.send(FindReferencesMessage { data }).await?
                    }
                    "textDocument/hover" => {
                        let (_, data) = get_message::<HoverRequest>(request).unwrap();
                        handler.send(HoverRequestMessage { data }).await?
                    }
                    "textDocument/documentSymbol" => {
                        let (_, data) = get_message::<DocumentSymbolRequest>(request).unwrap();
                        handler.send(DocumentSymbolMessage { data }).await?
                    }
                    "textDocument/documentHighlight" => {
                        let (_, data) = get_message::<DocumentHighlightRequest>(request).unwrap();
                        handler.send(DocumentHighlightMessage { data }).await?
                    }
                    "textDocument/semanticTokens/full" => {
                        let (_, data) = get_message::<SemanticTokensFullRequest>(request).unwrap();
                        handler
                            .send(SemanticTokensMessage {
                                text_document: data.text_document,
                                range: None,
                            })
                            .await?
                    }
                    "textDocument/semanticTokens/range" => {
                        let (_, data) = get_message::<SemanticTokensRangeRequest>(request).unwrap();
                        handler
                            .send(SemanticTokensMessage {
                                text_document: data.text_document,
                                range: Some(data.range),
                            })
                            .await?
                    }
                    _ => panic!("Unknown method: {:?}", request.method),
                };

                let response = make_response(req_id, &result);
                connection.sender.send(Message::Response(response))?;
            }
            Message::Notification(notification) => {
                debug!("Handling notification: {:#?}", notification.method);
                match notification.method.as_str() {
                    "textDocument/didOpen" => {
                        let data = get_notification::<DidOpenTextDocument>(notification).unwrap();

                        handler
                            .send(DocOpenNotification {
                                data: data.text_document,
                            })
                            .await?;
                    }
                    "textDocument/didChange" => {
                        let data = get_notification::<DidChangeTextDocument>(notification).unwrap();
                        handler.send(DocChangedNotification { data }).await?;
                    }
                    "textDocument/didSave" => {}
                    "textDocument/didClose" => {
                        let data = get_notification::<DidCloseTextDocument>(notification).unwrap();
                        handler.send(DocClosedNotification { data }).await?;
                    }
                    "$/cancelRequest" => debug!("Received cancel request - Ignoring"),
                    _ => panic!("Unknown notification: {:?}", notification.method),
                };
            }
            _ => (),
        };
    }

    Ok(())
}

fn make_response(id: RequestId, result: &LangServerResult) -> Response {
    match result {
        Ok(result) => {
            let result: Value = serde_json::from_str(result).unwrap();
            Response {
                id,
                result: Some(result),
                error: None,
            }
        }
        Err(err) => {
            let result: ResponseError = serde_json::from_str(err).unwrap();
            Response {
                id,
                result: None,
                error: Some(result),
            }
        }
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
