use std::collections::HashMap;

use crate::ast::{SyntaxKind, SyntaxNode};
use base::register::{RegisterKind, RegisterSize, Registers};
use rowan::GreenNode;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Alias {
    alias_map: HashMap<String, String>,
}

impl Alias {
    pub fn new() -> Self {
        Self {
            alias_map: Default::default(),
        }
    }

    pub fn add_alias(&mut self, node: &GreenNode) {
        let node = SyntaxNode::new_root(node.clone());

        let name = node
            .descendants_with_tokens()
            .find(|d| matches!(d.kind(), SyntaxKind::REGISTER_ALIAS))
            .map(|t| t.as_token().map(|t| t.to_string()))
            .flatten();

        let register = node
            .descendants_with_tokens()
            .find(|d| matches!(d.kind(), SyntaxKind::REGISTER))
            .map(|t| t.as_token().map(|t| t.to_string()))
            .flatten();

        if let (Some(name), Some(register)) = (name, register) {
            self.alias_map.insert(name, register);
        }
    }

    pub fn is_alias(&self, name: &str) -> bool {
        self.alias_map.contains_key(name)
    }

    pub fn get_register_for_alias(&self, name: &str) -> Option<&String> {
        self.alias_map.get(name)
    }

    pub fn get_alias_for_kind_size<'a>(
        &'a self,
        kind: RegisterKind,
        size: RegisterSize,
        registers: &'a (dyn Registers + 'a),
    ) -> impl Iterator<Item = &String> + 'a {
        self.alias_map.iter().filter_map(move |(k, v)| {
            (kind.contains(registers.get_kind(v)) && registers.get_size(v) == size).then(|| k)
        })
    }
}
