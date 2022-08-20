use std::collections::HashMap;

use crate::access::access_type;
use crate::tests::util;
use crate::{DocumentationMap, Instruction, InstructionTemplate, OperandAccessType};
use arch::registers::registers_for_architecture;
use base::Architecture;
use syntax::ast::{find_kind_index, SyntaxKind};

#[test]
fn test_access_types_simple() {
    let src = "addvl	sp, sp, #-2";
    let (parsed, alias) = util::parse_asm(src);

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

    assert_eq!(
        Some(OperandAccessType::Write),
        access_type(&first, &map, registers, &alias, Architecture::AArch64)
    );
    assert_eq!(
        Some(OperandAccessType::Read),
        access_type(&second, &map, registers, &alias, Architecture::AArch64)
    );
}

#[test]
fn test_access_types_with_alias() {
    let src = "test .req x1
addvl	test, sp, #-2";
    let (parsed, alias) = util::parse_asm(src);
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
        access_type(&first, &map, registers, &alias, Architecture::AArch64)
    );
    assert_eq!(
        Some(OperandAccessType::Read),
        access_type(&second, &map, registers, &alias, Architecture::AArch64)
    );
}
