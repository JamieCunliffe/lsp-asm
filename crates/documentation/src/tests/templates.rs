use arch::registers::registers_for_architecture;
use base::Architecture;
use syntax::ast::{find_kind_index, SyntaxKind};

use crate::templates::{find_correct_instruction_template, find_potential_instruction_templates};
use crate::tests::util;

#[test]
fn determine_instruction_from_template() {
    let line = "stp x29, x30, [sp, #32]";
    let (op, alias) = util::parse_asm(line);
    let op = op.first_child().unwrap();
    let instructions = vec![util::make_instruction()];

    assert_eq!(
        instructions[0].asm_template.get(5).unwrap(),
        find_correct_instruction_template(
            &op,
            &instructions,
            registers_for_architecture(&Architecture::AArch64),
            &alias,
            Architecture::AArch64
        )
        .unwrap(),
    );
}

#[test]
fn determine_instruction_from_template_with_alias() {
    let line = r#"reg_alias .req x29
stp reg_alias, x30, [sp, #32]"#;
    let (op, alias) = util::parse_asm(line);
    let instructions = vec![util::make_instruction()];

    assert_eq!(
        instructions[0].asm_template.get(5).unwrap(),
        find_correct_instruction_template(
            find_kind_index(&op, 0, SyntaxKind::INSTRUCTION)
                .unwrap()
                .as_node()
                .unwrap(),
            &instructions,
            registers_for_architecture(&Architecture::AArch64),
            &alias,
            Architecture::AArch64,
        )
        .unwrap(),
    );
}

#[test]
fn determine_instruction_from_template_with_equ() {
    let line = r#"number equ 32
stp x29, x30, [sp, #number]"#;
    let (op, alias) = util::parse_asm(line);
    let instructions = vec![util::make_instruction()];

    assert_eq!(
        instructions[0].asm_template.get(5).unwrap(),
        find_correct_instruction_template(
            find_kind_index(&op, 0, SyntaxKind::INSTRUCTION)
                .unwrap()
                .as_node()
                .unwrap(),
            &instructions,
            registers_for_architecture(&Architecture::AArch64),
            &alias,
            Architecture::AArch64,
        )
        .unwrap(),
    );
}

#[test]
fn determine_instruction_from_template_end_comment() {
    let line = "stp w29, w30, [sp, #32]    // Comment";
    let (op, alias) = util::parse_asm(line);
    let op = op.first_child().unwrap();
    let instructions = vec![util::make_instruction()];

    assert_eq!(
        instructions[0].asm_template.get(4).unwrap(),
        find_correct_instruction_template(
            &op,
            &instructions,
            registers_for_architecture(&Architecture::AArch64),
            &alias,
            Architecture::AArch64,
        )
        .unwrap(),
    );
}

#[test]
fn determine_potential_instructions_from_template() {
    let line = "stp x29, x30, [sp, #32]";
    let (op, alias) = util::parse_asm(line);
    let op = op.first_child().unwrap();

    let instructions = vec![util::make_instruction()];

    assert_eq!(
        vec![
            instructions[0].asm_template.get(3).unwrap(),
            instructions[0].asm_template.get(5).unwrap()
        ],
        find_potential_instruction_templates(
            &op,
            &instructions,
            registers_for_architecture(&Architecture::AArch64),
            &alias,
            Architecture::AArch64,
        ),
    );
}
