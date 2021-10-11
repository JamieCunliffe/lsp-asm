use std::collections::HashMap;
use std::sync::Arc;

use documentation::Instruction;
use syntax::ast::{find_kind_index, find_parent, SyntaxKind, SyntaxToken};

use crate::types::{CompletionItem, CompletionKind};

pub(super) fn ident_can_complete(
    ident: &str,
    token: &SyntaxToken,
    docs: Arc<HashMap<String, Vec<Instruction>>>,
) -> bool {
    || -> Option<bool> {
        let instruction = find_parent(token, SyntaxKind::INSTRUCTION)?;
        let op = find_kind_index(&instruction, 0, SyntaxKind::MNEMONIC)?.into_token()?;
        let instructions = docs.get(&op.text().to_lowercase())?;

        Some(instructions.iter().any(|instruction| {
            instruction.asm_template.iter().any(|t| {
                t.items
                    .iter()
                    .find_map(|op| {
                        (op.name == ident)
                            .then(|| op.completion_values.as_ref().map(|c| !c.is_empty()))
                    })
                    .flatten()
                    .unwrap_or(false)
            })
        }))
    }()
    .unwrap_or(false)
}

pub(super) fn complete_ident(
    ident: &str,
    token: &SyntaxToken,
    docs: Arc<HashMap<String, Vec<Instruction>>>,
) -> Vec<CompletionItem> {
    || -> Option<Vec<_>> {
        let instruction = find_parent(token, SyntaxKind::INSTRUCTION)?;
        let op = find_kind_index(&instruction, 0, SyntaxKind::MNEMONIC)?.into_token()?;
        let instructions = docs.get(&op.text().to_lowercase())?;

        Some(
            instructions
                .iter()
                .flat_map(|instruction| {
                    instruction.asm_template.iter().flat_map(|t| {
                        t.items
                            .iter()
                            .find(|op| op.name == ident)
                            .map(|t| t.completion_values.clone())
                            .flatten()
                            .unwrap_or_else(Default::default)
                    })
                })
                .map(to_completion)
                .collect(),
        )
    }()
    .unwrap_or_else(Default::default)
}

fn to_completion(value: String) -> CompletionItem {
    CompletionItem {
        text: value,
        details: String::from(""),
        documentation: None,
        kind: CompletionKind::Text,
    }
}

#[cfg(test)]
mod tests {
    use documentation::{DocumentationMap, Instruction, InstructionTemplate, OperandInfo};

    use super::*;
    use crate::{asm::parser::Parser, types::CompletionItem};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_mnemonic_completion() {
        let mut map = DocumentationMap::new();
        map.insert(
            "complete".into(),
            vec![Instruction {
                opcode: "COMPLETE".into(),
                header: None,
                architecture: None,
                description: "".into(),
                asm_template: vec![InstructionTemplate {
                    asm: vec![],
                    display_asm: "".into(),
                    items: vec![
                        OperandInfo {
                            name: "<pattern>".into(),
                            description: "".into(),
                            completion_values: Some(vec![
                                String::from("a"),
                                String::from("b"),
                                String::from("c"),
                            ]),
                        },
                        OperandInfo {
                            name: "<another_pattern>".into(),
                            description: "".into(),
                            completion_values: Some(vec![String::from("m")]),
                        },
                    ],
                }],
            }],
        );
        let map = Arc::new(map);

        let expected = vec![
            CompletionItem {
                text: "a".into(),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Text,
            },
            CompletionItem {
                text: "b".into(),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Text,
            },
            CompletionItem {
                text: "c".into(),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Text,
            },
        ];

        let data = r#"COMPLETE "#;
        let parser = Parser::from(data, &Default::default());
        let root = parser.tree();
        let token = find_kind_index(&root, 0, SyntaxKind::MNEMONIC)
            .unwrap()
            .into_token()
            .unwrap();
        assert_eq!(ident_can_complete("<pattern>", &token, map.clone()), true);
        assert_eq!(complete_ident("<pattern>", &token, map), expected);
    }
}
