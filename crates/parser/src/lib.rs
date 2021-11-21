use rowan::GreenNode;

#[macro_use]
extern crate log;

mod builder;
mod combinators;
pub mod config;
mod equ;
mod span;

pub use combinators::*;
use syntax::alias::Alias;

pub fn parse_asm(data: &str, config: &config::ParserConfig) -> ParsedData {
    combinators::parse(data, config)
}

pub struct ParsedData {
    pub root: GreenNode,
    pub alias: Alias,
}
