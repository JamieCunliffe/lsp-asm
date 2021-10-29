use std::collections::HashMap;

use syntax::ast::{find_kind_index, find_parent, SyntaxKind, SyntaxNode, SyntaxToken};

use crate::Instruction;

#[derive(Debug, Default)]
pub struct DocumentationMap {
    docs: HashMap<String, Vec<Instruction>>,
}

impl DocumentationMap {
    pub fn from(docs: HashMap<String, Vec<Instruction>>) -> Self {
        Self { docs }
    }

    pub fn get_from_token(&self, token: &SyntaxToken) -> Option<&Vec<Instruction>> {
        let instruction = find_parent(token, SyntaxKind::INSTRUCTION)?;
        let op = find_kind_index(&instruction, 0, SyntaxKind::MNEMONIC)?.into_token()?;
        self.get(op.text())
    }

    pub fn get_from_instruction_node(&self, node: &SyntaxNode) -> Option<&Vec<Instruction>> {
        let op = find_kind_index(node, 0, SyntaxKind::MNEMONIC)?.into_token()?;
        self.get(op.text())
    }

    pub fn get(&self, instruction: &str) -> Option<&Vec<Instruction>> {
        let instruction = instruction.to_lowercase();
        self.docs
            .get(&instruction)
            .or_else(|| self.get_without_attribute(&instruction))
    }

    fn get_without_attribute(&self, instruction: &str) -> Option<&Vec<Instruction>> {
        if let Some(index) = instruction.find('.') {
            let instruction = &instruction[..index];
            self.docs.get(instruction)
        } else {
            None
        }
    }

    pub fn instructions(&self) -> impl Iterator<Item = &Vec<Instruction>> {
        self.docs.values()
    }
}

impl<'de> serde::de::Deserialize<'de> for DocumentationMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let docs: HashMap<String, Vec<Instruction>> =
            serde::de::Deserialize::deserialize(deserializer)?;
        Ok(DocumentationMap::from(docs))
    }
}
