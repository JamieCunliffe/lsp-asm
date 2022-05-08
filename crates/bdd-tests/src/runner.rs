use std::convert::Infallible;
use std::str::FromStr;

use async_trait::async_trait;
use cucumber::{World, WorldInit};

use crate::command::LSPCommand;
use crate::file::FileUrl;
use crate::server::LSPServer;

mod command;
mod file;
mod position;
mod server;
mod steps;
mod util;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    LSPWorld::run("./features").await;
}

#[derive(WorldInit)]
pub struct LSPWorld {
    lsp: LSPServer,
    last_id: i32,
    last_cmd: LSPCommand,
    last_file: FileUrl,
}

#[async_trait(?Send)]
impl World for LSPWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Self::Error> {
        Ok(Default::default())
    }
}

impl core::fmt::Debug for LSPWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LSPWorld")
            .field("last_id", &self.last_id)
            .field("last_cmd", &self.last_cmd)
            .field("last_file", &self.last_file)
            .finish()
    }
}

impl Default for LSPWorld {
    fn default() -> Self {
        Self {
            lsp: LSPServer::new(),
            last_id: 0,
            last_cmd: LSPCommand::NoCommand,
            last_file: FileUrl::from_str("bdd-tests").unwrap(),
        }
    }
}
