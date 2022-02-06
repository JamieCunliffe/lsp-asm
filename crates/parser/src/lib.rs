use config::ParserConfig;
use rowan::GreenNode;

#[macro_use]
extern crate log;

mod builder;
mod combinators;
pub mod config;
mod equ;
mod include;
mod span;

pub use combinators::*;
use syntax::alias::Alias;

pub type LoadFileFn =
    fn(current_config: &ParserConfig, current_file: &str, filename: &str) -> Option<ParsedInclude>;

pub fn parse_asm(
    data: &str,
    config: &config::ParserConfig,
    file: Option<&str>,
    load: LoadFileFn,
) -> ParsedData {
    combinators::parse(data, config, file, load)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedData {
    pub root: GreenNode,
    pub alias: Alias,
    pub included_files: Vec<ParsedInclude>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedInclude {
    pub alias: Alias,
    pub root: GreenNode,
    pub included_files: Vec<ParsedInclude>,
    pub id: String,
    pub data: String,
}
