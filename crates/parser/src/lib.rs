use rowan::GreenNode;

#[macro_use]
extern crate log;

mod builder;
mod combinators;
pub mod config;
mod span;

pub use combinators::*;

pub fn parse_asm(data: &str, config: &config::ParserConfig) -> GreenNode {
    combinators::parse(data, config)
}

/// A register
#[derive(Debug, PartialEq)]
pub struct Register {
    /// A list of names that for this register, each name in this list is
    /// considered to be the same hardware register.
    pub names: &'static [&'static str],
}

impl Register {
    pub const fn new(names: &'static [&'static str]) -> Self {
        Self { names }
    }
}
