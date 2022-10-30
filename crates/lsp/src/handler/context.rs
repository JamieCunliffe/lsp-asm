use std::collections::HashMap;

use lsp_types::Url;
use parking_lot::RwLock;

use crate::asm::handler::AssemblyLanguageServerProtocol;
use crate::config::LSPConfig;
use crate::diagnostics::assembler_flags::AssemblerFlags;
use crate::diagnostics::compile_commands::CompileCommands;
use crate::diagnostics::Diagnostics;

#[derive(Default)]
pub struct Context {
    pub actors: RwLock<HashMap<Url, RwLock<AssemblyLanguageServerProtocol>>>,
    config: LSPConfig,
    pub commands: Option<Box<dyn Diagnostics + Send + Sync>>,
    pub root: String,
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
        }
    }

    pub fn config(&self) -> &LSPConfig {
        &self.config
    }
}
