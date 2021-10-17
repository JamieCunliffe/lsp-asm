use documentation::{Instruction, InstructionTemplate, OperandInfo};
use itertools::Itertools;
use lsp_types::{
    Documentation, ParameterInformation, ParameterLabel, SignatureHelp, SignatureInformation,
};
use rowan::TextSize;
use syntax::ast::{find_kind_index, find_parent, SyntaxKind, SyntaxNode, SyntaxToken};

use crate::asm::registers::registers_for_architecture;
use crate::documentation::is_potential_instruction_template;

use super::parser::Parser;

pub(super) fn get_signature_help(location: &TextSize, parser: &Parser) -> Option<SignatureHelp> {
    let token = parser.tree().token_at_offset(*location).left_biased()?;
    let instruction_node = find_parent(&token, SyntaxKind::INSTRUCTION)?.clone_for_update();
    let mnemonic =
        find_kind_index(&instruction_node, 0, SyntaxKind::MNEMONIC).map(|x| x.into_token())??;
    let docs = documentation::load_documentation(parser.architecture()).ok()?;
    let instructions = docs.get(mnemonic.text())?;
    let registers = registers_for_architecture(parser.architecture());

    let comma_index = get_comma_position(&instruction_node, &token);
    let instructions = instructions.iter().flat_map(|instruction| {
        instruction
            .asm_template
            .iter()
            .map(move |template| (instruction, template))
    });

    if let Some(token) = instruction_node
        .token_at_offset(token.text_range().start())
        .right_biased()
    {
        token.detach();
    }
    let active_signature =
        instructions
            .clone()
            .enumerate()
            .find_map(|(idx, (_instruction, template))| {
                is_potential_instruction_template(
                    &instruction_node,
                    template,
                    registers,
                    parser.alias(),
                )
                .then(|| idx)
            });

    let signatures = instructions
        .map(move |(instruction, template)| make_sig_help(instruction, template, comma_index))
        .collect_vec();

    let active_parameter = signatures
        .get(active_signature.unwrap_or_default())
        .map(|s: &SignatureInformation| s.active_parameter)
        .flatten();
    let active_signature = active_signature.map(|s| s as u32);
    Some(SignatureHelp {
        signatures,
        active_signature,
        active_parameter,
    })
}

fn make_sig_help(
    instruction: &Instruction,
    template: &InstructionTemplate,
    index: usize,
) -> SignatureInformation {
    SignatureInformation {
        label: template.display_asm.clone(),
        documentation: Some(Documentation::String(instruction.description.clone())),
        parameters: Some(make_parameters(&template.items, &template.display_asm)),
        active_parameter: Some(index as u32),
    }
}

fn get_comma_position(node: &SyntaxNode, current: &SyntaxToken) -> usize {
    node.descendants_with_tokens()
        .take_while(|d| d.as_token().map(|t| t != current).unwrap_or(true))
        .filter(|d| matches!(d.kind(), SyntaxKind::COMMA))
        .count()
}

fn make_parameters(operand_info: &[OperandInfo], display_asm: &str) -> Vec<ParameterInformation> {
    let operand_start = display_asm
        .chars()
        .take_while(|c| !c.is_whitespace())
        .count();

    let operands = display_asm[operand_start..].trim_start();

    let operands = operands
        .split(',')
        .map(|op| get_info_for_operands(op, operand_info));

    operands
        .map(|operand| {
            let label = operand
                .iter()
                .map(|operand| operand.name.clone())
                .collect::<String>();

            let documentation = operand
                .iter()
                .map(|operand| operand.description.clone())
                .collect::<Vec<_>>()
                .join("\n");

            ParameterInformation {
                label: ParameterLabel::Simple(label),
                documentation: Some(Documentation::String(documentation)),
            }
        })
        .collect()
}

fn get_info_for_operands<'op>(operand: &str, info: &'op [OperandInfo]) -> Vec<&'op OperandInfo> {
    let mut operand = operand;
    let mut ret = Vec::new();

    while let Some(start) = operand.find('<') {
        operand = &operand[start..];
        let end = operand.chars().take_while(|c| c != &'>').count() + 1;
        let part = &operand[..end];

        if let Some(info) = info.iter().find(|op| op.name == part) {
            ret.push(info);
        }
        operand = &operand[end..];
    }

    ret
}

#[cfg(test)]
mod tests {
    use documentation::OperandInfo;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_comma_index() {
        let data = "MNEMONIC ";
        let parser = crate::asm::parser::Parser::from(data, &Default::default());
        let nodes = parser.tree();
        let token = find_kind_index(&nodes, 0, SyntaxKind::WHITESPACE)
            .unwrap()
            .into_token()
            .unwrap();
        assert_eq!(get_comma_position(&nodes, &token), 0);

        let data = "MNEMONIC a, b";
        let parser = crate::asm::parser::Parser::from(data, &Default::default());
        let nodes = parser.tree();
        let token = find_kind_index(&nodes, 1, SyntaxKind::TOKEN)
            .unwrap()
            .into_token()
            .unwrap();
        assert_eq!(get_comma_position(&nodes, &token), 1);
    }

    #[test]
    fn combine_paramters() {
        // MNEMONIC <ab><x>, <test>
        // Should have two parameters `<ab><x>` and `<test>`
        let instruction = Instruction {
            opcode: Default::default(),
            header: Default::default(),
            architecture: Default::default(),
            description: Default::default(),
            asm_template: Default::default(),
        };
        let template = InstructionTemplate {
            asm: vec![],
            display_asm: "MNEMONIC <ab><x>, <test>, #<imm>".into(),
            items: vec![
                OperandInfo {
                    name: "<ab>".into(),
                    description: "ab".into(),
                    completion_values: Default::default(),
                },
                OperandInfo {
                    name: "<x>".into(),
                    description: "x".into(),
                    completion_values: Default::default(),
                },
                OperandInfo {
                    name: "<x>".into(),
                    description: "x".into(),
                    completion_values: Default::default(),
                },
                OperandInfo {
                    name: "<test>".into(),
                    description: "test".into(),
                    completion_values: Default::default(),
                },
                OperandInfo {
                    name: "<imm>".into(),
                    description: "imm".into(),
                    completion_values: Default::default(),
                },
            ],
        };
        let sig_info = make_sig_help(&instruction, &template, 0);

        let parameters = sig_info.parameters.unwrap();
        assert_eq!(parameters.len(), 3);

        let abx = parameters.get(0).unwrap();
        assert_eq!(abx.label, ParameterLabel::Simple("<ab><x>".into()));
        assert_eq!(
            abx.documentation,
            Some(Documentation::String("ab\nx".into()))
        );

        let test = parameters.get(1).unwrap();
        assert_eq!(test.label, ParameterLabel::Simple("<test>".into()));
        assert_eq!(
            test.documentation,
            Some(Documentation::String("test".into()))
        );

        let imm = parameters.get(2).unwrap();
        assert_eq!(imm.label, ParameterLabel::Simple("<imm>".into()));
        assert_eq!(imm.documentation, Some(Documentation::String("imm".into())));
    }
}
