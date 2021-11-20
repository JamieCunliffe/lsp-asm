use documentation::CompletionValue;
use itertools::Itertools;

use crate::types::{CompletionItem, CompletionKind};

use super::CompletionContext;

pub(super) fn ident_can_complete(ident: &str, context: &CompletionContext) -> bool {
    || -> Option<bool> {
        let instructions = context.docs.get_from_token(&context.token)?;

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

pub(super) fn complete_ident(ident: &str, context: &CompletionContext) -> Vec<CompletionItem> {
    || -> Option<Vec<_>> {
        let instructions = context.docs.get_from_token(&context.token)?;

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
                            .unwrap_or_default()
                    })
                })
                .flat_map(to_completion)
                .collect(),
        )
    }()
    .unwrap_or_default()
}

fn to_completion(value: CompletionValue) -> Vec<CompletionItem> {
    value.either(
        |value| {
            vec![CompletionItem {
                text: value,
                details: String::from(""),
                documentation: None,
                kind: CompletionKind::Text,
            }]
        },
        |range| {
            range
                .map(|value| CompletionItem {
                    text: value.to_string(),
                    details: String::from(""),
                    documentation: None,
                    kind: CompletionKind::Text,
                })
                .collect_vec()
        },
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use documentation::{DocumentationMap, Instruction, InstructionTemplate, OperandInfo};
    use syntax::ast::{find_kind_index, SyntaxKind};

    use super::*;
    use crate::{asm::parser::Parser, types::CompletionItem};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_mnemonic_completion() {
        let mut docs = HashMap::new();
        docs.insert(
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
                                CompletionValue::Left(String::from("a")),
                                CompletionValue::Left(String::from("b")),
                                CompletionValue::Left(String::from("c")),
                            ]),
                        },
                        OperandInfo {
                            name: "<another_pattern>".into(),
                            description: "".into(),
                            completion_values: Some(vec![CompletionValue::Left(String::from("m"))]),
                        },
                    ],
                    access_map: Default::default(),
                }],
            }],
        );
        let map = Arc::new(DocumentationMap::from(docs));

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
        let context = CompletionContext::new(&parser, token, map);

        assert_eq!(ident_can_complete("<pattern>", &context), true);
        assert_eq!(complete_ident("<pattern>", &context), expected);
    }

    #[test]
    fn test_mnemonic_completion_range() {
        let mut docs = HashMap::new();
        docs.insert(
            "complete".into(),
            vec![Instruction {
                opcode: "COMPLETE".into(),
                header: None,
                architecture: None,
                description: "".into(),
                asm_template: vec![InstructionTemplate {
                    asm: vec![],
                    display_asm: "".into(),
                    items: vec![OperandInfo {
                        name: "<imm>".into(),
                        description: "".into(),
                        completion_values: Some(vec![CompletionValue::Right(0..5)]),
                    }],
                    access_map: Default::default(),
                }],
            }],
        );
        let map = Arc::new(DocumentationMap::from(docs));

        let expected = vec![
            CompletionItem {
                text: "0".into(),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Text,
            },
            CompletionItem {
                text: "1".into(),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Text,
            },
            CompletionItem {
                text: "2".into(),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Text,
            },
            CompletionItem {
                text: "3".into(),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Text,
            },
            CompletionItem {
                text: "4".into(),
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
        let context = CompletionContext::new(&parser, token, map);

        assert_eq!(ident_can_complete("<imm>", &context), true);
        assert_eq!(complete_ident("<imm>", &context), expected);
    }
}
