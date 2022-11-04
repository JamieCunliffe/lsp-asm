use arch::registers::registers_for_architecture;
use base::Architecture;
use syntax::ast::{find_kind_index, SyntaxKind};

use crate::templates::{find_correct_instruction_template, find_potential_instruction_templates};
use crate::tests::util;
use crate::{Instruction, InstructionTemplate};

#[test]
fn determine_instruction_from_template() {
    let line = "stp x29, x30, [sp, #32]";
    let (op, alias) = util::parse_asm(line, base::FileType::Assembly);
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
    let (op, alias) = util::parse_asm(line, base::FileType::Assembly);
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
    let (op, alias) = util::parse_asm(line, base::FileType::Assembly);
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
    let (op, alias) = util::parse_asm(line, base::FileType::Assembly);
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
    let (op, alias) = util::parse_asm(line, base::FileType::Assembly);
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

#[test]
fn determine_instruction_template_with_label() {
    let line = "bl test";
    let (op, alias) = util::parse_asm(line, base::FileType::Assembly);
    let op = op.first_child().unwrap();

    let instructions = vec![util::make_label_instruction()];

    assert_eq!(
        instructions[0].asm_template.get(0).unwrap(),
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
fn determine_instruction_template_with_label_objdump() {
    let line = r#"0000000000000000 <a>:
000000000: 00 	bl test <a+0x4>"#;
    let (op, alias) = util::parse_asm(line, base::FileType::ObjDump);
    let op = op.first_child().unwrap();

    let instructions = vec![util::make_label_instruction()];

    assert_eq!(
        instructions[0].asm_template.get(0).unwrap(),
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

macro_rules! asm_match_template_test {
    ($name:ident, $template:literal, $asm:literal) => {
        #[test]
        fn $name() {
            let line = $asm;
            let (op, alias) = util::parse_asm(line, base::FileType::Assembly);
            let op = op.first_child().unwrap();

            let instructions = vec![Instruction {
                opcode: "".to_string(),
                header: None,
                architecture: None,
                description: "".to_string(),
                asm_template: vec![InstructionTemplate {
                    asm: vec![$template.to_string()],
                    display_asm: "".to_string(),
                    items: Default::default(),
                    access_map: Default::default(),
                }],
            }];

            assert_eq!(
                instructions[0].asm_template.get(0).unwrap(),
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
    };
}

asm_match_template_test!(
    determine_instruction_template_with_any_reg_size,
    "A  <gp|sp_64>, <gp|sp_64>, <gp_a>",
    "A x0, sp, w1"
);

asm_match_template_test!(
    determine_instruction_template_with_any_reg_size_sp,
    "A <scale_v>, <gp|sp_a>",
    "A z0, sp"
);

asm_match_template_test!(
    determine_instruction_template_with_any_reg_size_sp_gpx,
    "A <scale_v>, <gp|sp_a>",
    "A z0, x0"
);

asm_match_template_test!(
    determine_instruction_template_with_any_reg_size_sp_gpw,
    "A <scale_v>, <gp|sp_a>",
    "A z0, w0"
);
