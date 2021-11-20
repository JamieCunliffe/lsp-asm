use std::sync::Arc;

use documentation::DocumentationMap;
use itertools::Itertools;

use crate::types::{CompletionItem, CompletionKind};

pub(super) fn handle_mnemonic(docs: Arc<DocumentationMap>) -> Vec<CompletionItem> {
    let mut completions = docs
        .instructions()
        .flatten()
        .flat_map(|i| {
            i.asm_template.iter().map(move |t| CompletionItem {
                text: i.opcode.clone(),
                details: t.display_asm.clone(),
                documentation: Some(i.description.clone()),
                kind: CompletionKind::Mnemonic,
            })
        })
        .collect_vec();
    completions.sort();
    completions
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use documentation::{DocumentationMap, Instruction, InstructionTemplate};

    use super::*;
    use crate::types::CompletionItem;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_mnemonic_completion() {
        let mut docs = HashMap::new();
        docs.insert(
            "addvl".into(),
            vec![Instruction {
                opcode: "addvl".into(),
                header: None,
                architecture: None,
                description: "".into(),
                asm_template: vec![InstructionTemplate {
                    asm: vec!["ADDVL   <GP|SP_64>, <GP|SP_64>, #<imm>".into()],
                    display_asm: "".into(),
                    items: vec![],
                    access_map: Default::default(),
                }],
            }],
        );
        docs.insert(
            "cnt".into(),
            vec![Instruction {
                opcode: "cnt".into(),
                header: None,
                architecture: None,
                description: "Documentation".into(),
                asm_template: vec![InstructionTemplate {
                    asm: vec!["CNT  <SIMD_V>, <SIMD_V>".into()],
                    display_asm: "CNT  <Vd>, <Vn>".into(),
                    items: vec![],
                    access_map: Default::default(),
                }],
            }],
        );

        let expected = vec![
            CompletionItem {
                text: "addvl".into(),
                details: "".into(),
                documentation: Some("".into()),
                kind: crate::types::CompletionKind::Mnemonic,
            },
            CompletionItem {
                text: "cnt".into(),
                details: "CNT  <Vd>, <Vn>".into(),
                documentation: Some("Documentation".into()),
                kind: crate::types::CompletionKind::Mnemonic,
            },
        ];

        assert_eq!(
            handle_mnemonic(Arc::new(DocumentationMap::from(docs))),
            expected
        );
    }
}
