use std::sync::atomic::AtomicI32;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use lsp_asm::config::LSPConfig;
use lsp_server::{Connection, Message, Notification, Request};
use lsp_types::notification::{DidOpenTextDocument, Exit, Initialized, PublishDiagnostics};
use lsp_types::request::{Initialize, Shutdown};
use lsp_types::{
    DidOpenTextDocumentParams, InitializeParams, InitializedParams, PublishDiagnosticsParams,
    TextDocumentItem, Url,
};
use serde_json::Value;

pub struct LSPServer {
    init: bool,
    connection: Arc<Connection>,
    client: Arc<Connection>,
    req_id: AtomicI32,
    lsp_thread: Option<JoinHandle<()>>,
    receved_messages: Arc<Mutex<Vec<Message>>>,
}

impl LSPServer {
    pub fn new() -> Self {
        let (connection, client) = Connection::memory();
        let connection = Arc::new(connection);
        let client = Arc::new(client);
        Self {
            init: false,
            connection,
            client,
            req_id: AtomicI32::new(0),
            lsp_thread: None,
            receved_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn init(&mut self, config: LSPConfig, root_uri: Option<String>) {
        #[allow(deprecated)]
        let client_init = InitializeParams {
            process_id: None,
            root_path: None,
            root_uri: root_uri
                .as_ref()
                .map(|uri| Url::from_directory_path(uri).expect("Failed to uri")),
            initialization_options: Some(
                serde_json::to_value(config).expect("Failed to serilize config"),
            ),
            capabilities: lsp_types::ClientCapabilities {
                ..Default::default()
            },
            trace: None,
            workspace_folders: None,
            client_info: None,
            locale: None,
        };
        self.init = true;

        // Send the initialization request
        self.send_request::<Initialize>(client_init);

        // Queue up the initilized notification so that the call to initialize has all the messages it needs
        self.send_notification::<Initialized>(InitializedParams {});

        let server_capabilities =
            serde_json::to_value(&lsp_asm::capabilities::get_server_capabilities())
                .expect("Failed to encode server capabilities");
        let initialization_params = self
            .connection
            .initialize(server_capabilities)
            .expect("Failed to initialize server");
        let initialization_params = serde_json::from_value(initialization_params).unwrap();

        // Start the lsp loop on a thread so it lives.
        let server_connection = self.connection.clone();
        self.lsp_thread = Some(std::thread::spawn(move || {
            lsp_asm::lsp::lsp_loop(server_connection, initialization_params)
                .expect("lsp loop failed");
        }));

        let client = self.client.clone();
        let messages = self.receved_messages.clone();
        std::thread::spawn(move || {
            let client = client.clone();
            let messages = messages;

            while let Ok(msg) = client.receiver.recv() {
                if let Ok(mut messages) = messages.lock() {
                    messages.push(msg);
                } else {
                    panic!("Lock failed when adding: {:#?}", msg);
                }
            }
        });
    }

    pub fn open_file(&self, uri: Url, text: &str) {
        assert!(self.init);

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: String::from("asm"),
                version: 0,
                text: text.into(),
            },
        };

        self.send_notification::<DidOpenTextDocument>(open_params);
        loop {
            if !self.diagnostics_for_file(&uri).is_empty() {
                return;
            }
        }
    }

    pub fn send_shutdown(&self) {
        // Send the shut down request
        self.send_request::<Shutdown>(());
        // Queue up the exit notification
        self.send_notification::<Exit>(());
    }

    pub fn send_request<R>(&self, params: R::Params) -> i32
    where
        R: lsp_types::request::Request,
    {
        assert!(self.init);

        let id = self
            .req_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        self.client
            .sender
            .send(Message::Request(Request::new(
                id.into(),
                R::METHOD.into(),
                params,
            )))
            .unwrap_or_else(|_| panic!("Failed to send {} request", R::METHOD));

        id
    }

    pub fn wait_for_response_for_id(&self, id: i32) -> Value {
        assert!(self.init);
        loop {
            if let Ok(messages) = self.receved_messages.lock() {
                if let Some(resp) = messages
                    .iter()
                    .find(|m| matches!(m, Message::Response(resp) if resp.id == id.into()))
                {
                    match resp {
                        Message::Response(resp) => return resp.result.as_ref().unwrap().clone(),
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    pub fn diagnostics_for_file(&self, file: &Url) -> Vec<PublishDiagnosticsParams> {
        assert!(self.init);

        if let Ok(messages) = self.receved_messages.lock() {
            messages
                .iter()
                .filter_map(|m| match m {
                    Message::Notification(not) if not.method == <PublishDiagnostics as lsp_types::notification::Notification>::METHOD => {
                        let x: Option<PublishDiagnosticsParams> = serde_json::from_value(not.params.clone()).ok();
                        match x {
                            Some(p) if p.uri == *file => Some(p),
                            _ => None,
                        }
                    },
                    _ => None,
                }).collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    }

    pub fn send_notification<N>(&self, params: N::Params)
    where
        N: lsp_types::notification::Notification,
    {
        assert!(self.init);

        self.client
            .sender
            .send(Message::Notification(Notification::new(
                N::METHOD.into(),
                params,
            )))
            .unwrap_or_else(|_| panic!("Failed to send {} notification", N::METHOD));
    }
}

impl Default for LSPServer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LSPServer {
    fn drop(&mut self) {
        self.send_shutdown();
        self.lsp_thread
            .take()
            .unwrap()
            .join()
            .expect("Failed to join thread");
    }
}
