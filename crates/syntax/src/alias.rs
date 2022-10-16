use std::collections::HashMap;

use crate::ast::{SyntaxKind, SyntaxNode};
use base::register::{RegisterKind, RegisterSize, Registers};
use itertools::Itertools;
use rowan::GreenNode;

#[derive(Debug, Clone, PartialEq)]
enum Kind {
    Register(String),
    Constant(String),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Alias {
    alias_map: HashMap<String, Kind>,
}

impl Alias {
    pub fn new() -> Self {
        Self {
            alias_map: Default::default(),
        }
    }

    pub fn merge(&mut self, other: &Alias) {
        for (k, v) in &other.alias_map {
            self.alias_map.insert(k.clone(), v.clone());
        }
    }

    pub fn add_alias(&mut self, node: &GreenNode) {
        let node = SyntaxNode::new_root(node.clone());

        let name = node
            .descendants_with_tokens()
            .find(|d| matches!(d.kind(), SyntaxKind::REGISTER_ALIAS))
            .and_then(|t| t.as_token().map(|t| t.to_string()));

        let register = node
            .descendants_with_tokens()
            .find(|d| matches!(d.kind(), SyntaxKind::REGISTER))
            .and_then(|t| t.as_token().map(|t| t.to_string()));

        if let (Some(name), Some(register)) = (name, register) {
            self.alias_map.insert(name, Kind::Register(register));
        }
    }

    pub fn add_equ(&mut self, node: &GreenNode) {
        let node = SyntaxNode::new_root(node.clone());

        let name = node
            .descendants_with_tokens()
            .find(|d| matches!(d.kind(), SyntaxKind::NAME))
            .and_then(|t| t.as_token().map(|t| t.to_string()));

        let expr = node
            .descendants_with_tokens()
            .find(|d| matches!(d.kind(), SyntaxKind::EXPR))
            .and_then(|n| n.into_node())
            .map(|n| {
                n.descendants_with_tokens()
                    .filter_map(|t| t.as_token().map(|t| t.to_string()))
                    .join("")
            })
            .unwrap_or_default();

        if let Some(name) = name {
            self.alias_map.insert(name, Kind::Constant(expr));
        }
    }

    pub fn get_kind(&self, token: &str) -> Option<SyntaxKind> {
        self.alias_map.get(token).map(|k| match k {
            Kind::Register(_) => SyntaxKind::REGISTER_ALIAS,
            Kind::Constant(_) => SyntaxKind::CONSTANT,
        })
    }

    pub fn get_register_for_alias(&self, name: &str) -> Option<&String> {
        self.alias_map.get(name).and_then(|val| match val {
            Kind::Register(r) => Some(r),
            Kind::Constant(_) => None,
        })
    }

    pub fn get_constant_for_token(&self, name: &str) -> Option<&String> {
        self.alias_map.get(name).and_then(|val| match val {
            Kind::Register(_) => None,
            Kind::Constant(expr) => Some(expr),
        })
    }

    pub fn get_alias_for_kind_size<'a>(
        &'a self,
        kind: RegisterKind,
        size: RegisterSize,
        registers: &'a (dyn Registers + 'a),
    ) -> impl Iterator<Item = &String> + 'a {
        self.alias_map.iter().filter_map(move |(k, v)| match v {
            Kind::Register(v) => {
                (kind.contains(registers.get_kind(v)) && registers.get_size(v) == size).then_some(k)
            }
            _ => None,
        })
    }
}
