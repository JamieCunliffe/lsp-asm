use std::collections::HashMap;

use lsp_types::Url;
use parking_lot::RwLock;

use crate::asm::handler::AssemblyLanguageServerProtocol;
use crate::asm::parser::Parser;
use crate::config::LSPConfig;
use crate::diagnostics::assembler_flags::AssemblerFlags;
use crate::diagnostics::compile_commands::CompileCommands;
use crate::diagnostics::Diagnostics;

use super::file_graph::FileGraph;

#[derive(Default)]
pub struct Context {
    pub actors: RwLock<HashMap<Url, RwLock<AssemblyLanguageServerProtocol>>>,
    config: LSPConfig,
    pub commands: Option<Box<dyn Diagnostics + Send + Sync>>,
    pub root: String,
    pub file_graph: RwLock<FileGraph>,
}

impl Context {
    pub fn new(config: LSPConfig, root: String) -> Self {
        info!("Initializing workspace: {}", root);

        let commands: Option<Box<dyn Diagnostics + Send + Sync>> =
            if let Some(compile_commands) = CompileCommands::new(&root) {
                Some(Box::new(compile_commands))
            } else if let Some(flags) = AssemblerFlags::new(&root) {
                Some(Box::new(flags))
            } else {
                None
            };

        Self {
            actors: RwLock::new(HashMap::new()),
            config,
            commands,
            root,
            file_graph: Default::default(),
        }
    }

    pub fn config(&self) -> &LSPConfig {
        &self.config
    }

    pub fn add_include(&self, parent: String, child: String) {
        self.file_graph.write().insert(&parent, &[&child]);
    }

    pub fn add_actors(&self, new_actors: Vec<(Url, RwLock<AssemblyLanguageServerProtocol>)>) {
        if !new_actors.is_empty() {
            let mut actors = self.actors.write();
            for (url, actor) in new_actors {
                actors.entry(url).or_insert(actor);
            }
        }
    }

    pub(crate) fn related_parsers<I, T, F>(&self, include_self: bool, uri: Url, f: F) -> Vec<T>
    where
        F: Fn(&Parser) -> I,
        I: Iterator<Item = T>,
    {
        let files = self.file_graph.read().get_related_files(uri.as_ref());
        let mut parsers = Vec::with_capacity(files.len());

        let handle_file = |parsers: &mut Vec<_>, file: &Url| {
            let actors = self.actors.read();
            let actor = actors.get(file);
            if let Some(actor) = actor {
                let actor = actor.read();
                let parser = actor.parser();
                let items = f(parser);
                parsers.extend(items);
            } else {
                warn!("Expected {file} not found");
            }
        };

        if include_self {
            handle_file(&mut parsers, &uri);
        }
        for file in files {
            handle_file(&mut parsers, &Url::parse(&file).unwrap());
        }

        parsers
    }
}
