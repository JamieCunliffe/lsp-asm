use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::asm::handler::AssemblyLanguageServerProtocol;
use crate::types::DocumentPosition;

use super::LanguageServerProtocol;

use actix::{Actor, Context, Handler, Message};

use lsp_server::ResponseError;

use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DocumentHighlightParams,
    DocumentSymbolParams, GotoDefinitionParams, HoverParams, Range, ReferenceParams,
    TextDocumentIdentifier, TextDocumentItem, Url,
};

pub(crate) type LangServerResult = Result<String, String>;

pub struct LangServerHandler {
    actors: HashMap<Url, Box<dyn LanguageServerProtocol>>,
}

impl Default for LangServerHandler {
    fn default() -> Self {
        Self {
            actors: HashMap::new(),
        }
    }
}

impl Actor for LangServerHandler {
    type Context = Context<Self>;
}

pub(crate) fn make_result<T>(result: &Result<T, ResponseError>) -> LangServerResult
where
    T: serde::Serialize,
{
    match result {
        Ok(result) => Ok(serde_json::to_value(&result).unwrap().to_string()),
        Err(result) => Err(serde_json::to_value(&result).unwrap().to_string()),
    }
}

#[derive(Message)]
#[rtype(result = "LangServerResult")]
pub struct GotoDefinitionMessage {
    pub data: GotoDefinitionParams,
}

#[derive(Message)]
#[rtype(result = "LangServerResult")]
pub struct FindReferencesMessage {
    pub data: ReferenceParams,
}

#[derive(Message)]
#[rtype(result = "LangServerResult")]
pub struct HoverRequestMessage {
    pub data: HoverParams,
}

#[derive(Message)]
#[rtype(result = "LangServerResult")]
pub struct DocumentSymbolMessage {
    pub data: DocumentSymbolParams,
}

#[derive(Message)]
#[rtype(result = "LangServerResult")]
pub struct SemanticTokensMessage {
    pub text_document: TextDocumentIdentifier,
    pub range: Option<Range>,
}

#[derive(Message)]
#[rtype(result = "LangServerResult")]
pub struct DocumentHighlightMessage {
    pub data: DocumentHighlightParams,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DocOpenNotification {
    pub data: TextDocumentItem,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DocChangedNotification {
    pub data: DidChangeTextDocumentParams,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DocClosedNotification {
    pub data: DidCloseTextDocumentParams,
}

impl Handler<GotoDefinitionMessage> for LangServerHandler {
    type Result = LangServerResult;

    fn handle(&mut self, msg: GotoDefinitionMessage, _ctx: &mut Self::Context) -> Self::Result {
        let handler = self
            .actors
            .get(&msg.data.text_document_position_params.text_document.uri)
            .unwrap();

        let result = handler.goto_definition(DocumentPosition::from(
            msg.data.text_document_position_params,
        ));
        make_result(&result)
    }
}

impl Handler<FindReferencesMessage> for LangServerHandler {
    type Result = LangServerResult;

    fn handle(&mut self, msg: FindReferencesMessage, _ctx: &mut Self::Context) -> Self::Result {
        let handler = self
            .actors
            .get(&msg.data.text_document_position.text_document.uri)
            .unwrap();

        let result = handler.find_references(
            DocumentPosition::from(msg.data.text_document_position),
            msg.data.context.include_declaration,
        );

        make_result(&result)
    }
}

impl Handler<HoverRequestMessage> for LangServerHandler {
    type Result = LangServerResult;

    fn handle(&mut self, msg: HoverRequestMessage, _ctx: &mut Self::Context) -> Self::Result {
        let handler = self
            .actors
            .get(&msg.data.text_document_position_params.text_document.uri)
            .unwrap();

        let result = handler.hover(DocumentPosition::from(
            msg.data.text_document_position_params,
        ));

        make_result(&result)
    }
}

impl Handler<DocumentSymbolMessage> for LangServerHandler {
    type Result = LangServerResult;

    fn handle(&mut self, msg: DocumentSymbolMessage, _ctx: &mut Self::Context) -> Self::Result {
        let handler = self.actors.get(&msg.data.text_document.uri).unwrap();

        let result = handler.document_symbols();

        make_result(&result)
    }
}

impl Handler<SemanticTokensMessage> for LangServerHandler {
    type Result = LangServerResult;

    fn handle(&mut self, msg: SemanticTokensMessage, _ctx: &mut Self::Context) -> Self::Result {
        let handler = self.actors.get(&msg.text_document.uri).unwrap();

        let result = handler.get_semantic_tokens(msg.range);

        make_result(&result)
    }
}

impl Handler<DocumentHighlightMessage> for LangServerHandler {
    type Result = LangServerResult;

    fn handle(&mut self, msg: DocumentHighlightMessage, _ctx: &mut Self::Context) -> Self::Result {
        let handler = self
            .actors
            .get(&msg.data.text_document_position_params.text_document.uri)
            .unwrap();

        let result = handler.document_highlight(DocumentPosition::from(
            msg.data.text_document_position_params,
        ));

        make_result(&result)
    }
}

impl Handler<DocOpenNotification> for LangServerHandler {
    type Result = ();

    fn handle(&mut self, msg: DocOpenNotification, _ctx: &mut Self::Context) -> Self::Result {
        let uri = msg.data.uri;
        let text = msg.data.text;

        let actor = match msg.data.language_id.to_lowercase().as_str() {
            "asm" => AssemblyLanguageServerProtocol::new(&text, uri.clone()),
            "assembly" => AssemblyLanguageServerProtocol::new(&text, uri.clone()),
            _ => panic!("Unknown language: {}", msg.data.language_id),
        };

        self.actors.insert(uri, Box::new(actor));
    }
}

impl Handler<DocChangedNotification> for LangServerHandler {
    type Result = ();

    fn handle(&mut self, msg: DocChangedNotification, _ctx: &mut Self::Context) -> Self::Result {
        let uri = msg.data.text_document.uri;
        let change_params = msg.data.content_changes;
        if change_params.len() != 1 {
            panic!("Unexpected document changed parameters");
        }

        let text = &change_params.first().unwrap().text;
        self.actors.get_mut(&uri).unwrap().update(text);
    }
}

impl Handler<DocClosedNotification> for LangServerHandler {
    type Result = ();

    fn handle(&mut self, msg: DocClosedNotification, _ctx: &mut Self::Context) -> Self::Result {
        let uri = msg.data.text_document.uri;

        if let Entry::Occupied(entry) = self.actors.entry(uri) {
            entry.remove_entry();
        }
    }
}
