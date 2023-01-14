#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[macro_use]
extern crate log;

use lsp_asm::capabilities::get_server_capabilities;

use lsp_server::Connection;

use std::error::Error;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    pretty_env_logger::init_timed();
    debug!("Starting lsp-asm");

    let (connection, io_threads) = Connection::stdio();
    let connection = Arc::new(connection);

    let capabilities = get_server_capabilities();

    let server_capabilities = serde_json::to_value(capabilities).unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;
    let initialization_params = serde_json::from_value(initialization_params)?;
    lsp_asm::lsp::lsp_loop(connection, initialization_params)?;
    io_threads.join()?;

    debug!("Shutting server down");
    Ok(())
}
