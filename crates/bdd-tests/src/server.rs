use std::panic::Location;
use std::sync::atomic::AtomicI32;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use lsp_asm::config::LSPConfig;
use lsp_asm::handler::ext::{FileStatsParams, FileStatsResult};
use lsp_server::{Connection, Message, Notification, Request, ResponseError};
use lsp_types::notification::{
    DidCloseTextDocument, DidOpenTextDocument, Exit, Initialized, PublishDiagnostics,
};
use lsp_types::request::{Initialize, Shutdown};
use lsp_types::{
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializedParams,
    PublishDiagnosticsParams, TextDocumentIdentifier, TextDocumentItem, Url,
};
use parking_lot::RwLock;
use serde_json::Value;

pub struct LSPServer {
    init: bool,
    connection: Arc<Connection>,
    client: Arc<Connection>,
    req_id: AtomicI32,
    lsp_thread: Option<JoinHandle<()>>,
    receved_messages: Arc<RwLock<Vec<Message>>>,
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
            receved_messages: Default::default(),
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
            serde_json::to_value(lsp_asm::capabilities::get_server_capabilities())
                .expect("Failed to encode server capabilities");
        let initialization_params = self
            .connection
            .initialize(server_capabilities)
            .expect("Failed to initialize server");
        let initialization_params = serde_json::from_value(initialization_params).unwrap();

        // Start the lsp loop on a thread so it lives.
        let server_connection = self.connection.clone();
        self.lsp_thread = Some(
            std::thread::Builder::new()
                .name(String::from("lsp_loop"))
                .spawn(move || {
                    lsp_asm::lsp::lsp_loop(server_connection, initialization_params)
                        .expect("lsp loop failed");
                })
                .unwrap(),
        );

        let client = self.client.clone();
        let messages = self.receved_messages.clone();
        std::thread::Builder::new()
            .name(String::from("Receiver"))
            .spawn(move || {
                let client = client.clone();
                let messages = messages;

                while let Ok(msg) = client.receiver.recv() {
                    let mut messages = messages.write();
                    messages.push(msg);
                }
            })
            .unwrap();
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

    pub fn close_file(&self, uri: Url) {
        assert!(self.init);

        let close_params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
        };

        self.send_notification::<DidCloseTextDocument>(close_params);
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

    #[track_caller]
    pub fn wait_for_response_for_id(&self, id: i32) -> Value {
        assert!(self.init);
        let start = Instant::now();
        loop {
            {
                let messages = self.receved_messages.read();
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
            if Instant::now() - start > Duration::from_secs(5) {
                panic!(
                    r#"Time out waiting for message id: {id} from {}
Messages held: {:#?}"#,
                    Location::caller(),
                    self.receved_messages.read()
                );
            }
        }
    }

    #[track_caller]
    pub fn wait_for_error_for_id(&self, id: i32) -> ResponseError {
        assert!(self.init);
        let start = Instant::now();
        loop {
            {
                let messages = self.receved_messages.read();
                if let Some(resp) = messages
                    .iter()
                    .find(|m| matches!(m, Message::Response(resp) if resp.id == id.into()))
                {
                    match resp {
                        Message::Response(resp) => return resp.error.as_ref().unwrap().clone(),
                        _ => unreachable!(),
                    }
                }
            }
            if Instant::now() - start > Duration::from_secs(5) {
                panic!(
                    r#"Time out waiting for message id: {id} from {}
Messages held: {:#?}"#,
                    Location::caller(),
                    self.receved_messages.read()
                );
            }
        }
    }

    pub fn diagnostics_for_file(&self, file: &Url) -> Vec<PublishDiagnosticsParams> {
        assert!(self.init);

        let messages = self.receved_messages.read();
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

    pub fn wait_for_file_version(&self, url: Url, version: i32) {
        loop {
            let id = self.send_request::<lsp_asm::handler::ext::FileStats>(FileStatsParams {
                url: url.clone(),
            });

            let resp = self.wait_for_response_for_id(id);
            if let Ok(resp) = serde_json::from_value::<FileStatsResult>(resp) {
                if resp.version == version as u32 {
                    break;
                }
            }
        }
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
