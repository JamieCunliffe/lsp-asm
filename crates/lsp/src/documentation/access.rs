use base::register::Registers;
use documentation::{DocumentationMap, OperandAccessType};
use syntax::{
    alias::Alias,
    ast::{find_parent, SyntaxKind, SyntaxToken},
};

pub(crate) fn access_type(
    token: &SyntaxToken,
    docs: &DocumentationMap,
    registers: &dyn Registers,
    alias: &Alias,
) -> Option<OperandAccessType> {
    let instruction = find_parent(token, SyntaxKind::INSTRUCTION)?;

    let (index, _) = instruction
        .descendants_with_tokens()
        .filter(|c| {
            matches!(
                c.kind(),
                SyntaxKind::REGISTER
                    | SyntaxKind::NUMBER
                    | SyntaxKind::FLOAT
                    | SyntaxKind::TOKEN
                    | SyntaxKind::REGISTER_ALIAS
            )
        })
        .enumerate()
        .find(|(_, t)| t.as_token().map(|t| t == token).unwrap_or(false))?;

    let instructions = docs.get_from_instruction_node(&instruction)?;

    let template =
        super::find_correct_instruction_template(&instruction, instructions, registers, alias)?;

    template.access_map.get(index).cloned()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::access_type;
    use crate::asm::parser::Parser;
    use crate::asm::registers::registers_for_architecture;
    use base::Architecture;
    use documentation::{DocumentationMap, Instruction, InstructionTemplate, OperandAccessType};
    use parser::ParsedData;
    use syntax::ast::{find_kind_index, SyntaxKind, SyntaxNode};

    #[test]
    fn test_access_types_simple() {
        let src = "addvl	sp, sp, #-2";
        let ParsedData { root, .. } = parser::parse_asm(
            src,
            &Parser::config_from_arch(&Architecture::AArch64),
            None,
            |_, _, _| None,
        );
        let parsed = SyntaxNode::new_root(root);

        let mut map = HashMap::new();
        map.insert(
            "addvl".into(),
            vec![Instruction {
                opcode: "addvl".into(),
                header: None,
                architecture: None,
                description: "".into(),
                asm_template: vec![InstructionTemplate {
                    asm: vec!["ADDVL   <gp|sp_64>, <gp|sp_64>, #<imm>".into()],
                    display_asm: "".into(),
                    items: vec![],
                    access_map: vec![OperandAccessType::Write, OperandAccessType::Read],
                }],
            }],
        );
        let map = DocumentationMap::from(map);
        let registers = registers_for_architecture(&Architecture::AArch64);
        let first = find_kind_index(&parsed, 0, SyntaxKind::REGISTER)
            .unwrap()
            .into_token()
            .unwrap();
        let second = find_kind_index(&parsed, 1, SyntaxKind::REGISTER)
            .unwrap()
            .into_token()
            .unwrap();
        let alias = Default::default();
        assert_eq!(
            Some(OperandAccessType::Write),
            access_type(&first, &map, registers, &alias)
        );
        assert_eq!(
            Some(OperandAccessType::Read),
            access_type(&second, &map, registers, &alias)
        );
    }

    #[test]
    fn test_access_types_with_alias() {
        let src = "test .req x1
addvl	test, sp, #-2";
        let ParsedData { root, alias, .. } = parser::parse_asm(
            src,
            &Parser::config_from_arch(&Architecture::AArch64),
            None,
            |_, _, _| None,
        );
        let parsed = SyntaxNode::new_root(root);
        let parsed = find_kind_index(&parsed, 0, SyntaxKind::INSTRUCTION)
            .unwrap()
            .into_node()
            .unwrap();

        let mut map = HashMap::new();
        map.insert(
            "addvl".into(),
            vec![Instruction {
                opcode: "addvl".into(),
                header: None,
                architecture: None,
                description: "".into(),
                asm_template: vec![InstructionTemplate {
                    asm: vec!["ADDVL   <gp|sp_64>, <gp|sp_64>, #<imm>".into()],
                    display_asm: "".into(),
                    items: vec![],
                    access_map: vec![OperandAccessType::Write, OperandAccessType::Read],
                }],
            }],
        );
        let map = DocumentationMap::from(map);
        let registers = registers_for_architecture(&Architecture::AArch64);
        let first = find_kind_index(&parsed, 0, SyntaxKind::REGISTER_ALIAS)
            .unwrap()
            .into_token()
            .unwrap();
        let second = find_kind_index(&parsed, 0, SyntaxKind::REGISTER)
            .unwrap()
            .into_token()
            .unwrap();

        assert_eq!(
            Some(OperandAccessType::Write),
            access_type(&first, &map, registers, &alias)
        );
        assert_eq!(
            Some(OperandAccessType::Read),
            access_type(&second, &map, registers, &alias)
        );
    }
}
