use itertools::Itertools;

use base::register::{RegisterKind, Registers};
use base::Architecture;
use documentation::registers::DOC_REGISTERS;
use documentation::{Instruction, InstructionTemplate};
use lazy_static::lazy_static;
use parser::config::ParserConfig;
use syntax::ast::{SyntaxElement, SyntaxKind, SyntaxNode};

lazy_static! {
    static ref TEMPLATE_CONFIG: ParserConfig = ParserConfig {
        comment_start: String::from("//"),
        architecture: Architecture::AArch64,
        file_type: Default::default(),
        registers: Some(&documentation::registers::DOCUMENTATION_REGISTERS),
    };
}
/// Finds the instruction template that matches the given node, this will
/// require an exact match to be found otherwise None is returned
pub fn find_correct_instruction_template<'a>(
    node: &SyntaxNode,
    instructions: &'a [Instruction],
    lookup: &Option<impl Registers>,
) -> Option<&'a InstructionTemplate> {
    instructions
        .iter()
        .flat_map(|x| &x.asm_template)
        .find(|template| {
            template
                .asm
                .iter()
                .any(|template| check_template(template, node, true, lookup))
        })
}

/// Finds the instruction templates that match the given node, this will return
/// templates that are a partial match i.e. if the template is longer than the
/// input but up to that point it matches, then it will be included.
#[allow(unused)]
pub fn find_potential_instruction_templates<'a>(
    node: &SyntaxNode,
    instructions: &'a [Instruction],
    lookup: &Option<impl Registers>,
) -> Vec<&'a InstructionTemplate> {
    instructions
        .iter()
        .flat_map(|x| &x.asm_template)
        .filter(|template| {
            template
                .asm
                .iter()
                .any(|template| check_template(template, node, false, lookup))
        })
        .collect_vec()
}

/// Given an an instruction template, find the instruction that it belongs to.
pub fn instruction_from_template<'a>(
    instructions: &'a [Instruction],
    template: &InstructionTemplate,
) -> Option<&'a Instruction> {
    instructions
        .iter()
        .find(|x| x.asm_template.iter().any(|x| x == template))
}

fn check_template(
    template: &str,
    node: &SyntaxNode,
    exact: bool,
    lookup: &Option<impl Registers>,
) -> bool {
    // Some of the registers have a + in the name, replace that + with
    // ADD so that the parser doesn't split it up.
    let parsed_template = SyntaxNode::new_root(parser::parse_asm(
        template.replace("+", "ADD").as_str(),
        &TEMPLATE_CONFIG,
    ));

    let parsed_template = parsed_template.first_child().unwrap();
    let filtered = node
        .descendants_with_tokens()
        .filter(|c| {
            !matches!(
                c.kind(),
                SyntaxKind::WHITESPACE | SyntaxKind::METADATA | SyntaxKind::COMMENT
            )
        })
        .collect_vec();

    let filtered_parsed = parsed_template
        .descendants_with_tokens()
        .filter(|c| !matches!(c.kind(), SyntaxKind::WHITESPACE | SyntaxKind::METADATA))
        .collect_vec();

    (!exact || filtered.len() == filtered_parsed.len())
        && filtered
            .iter()
            .zip(filtered_parsed.iter())
            .all(|x| node_or_token_match(x, lookup))
}

fn node_or_token_match(
    elements: (&SyntaxElement, &SyntaxElement),
    lookup: &Option<impl Registers>,
) -> bool {
    match elements {
        (a, t) if a.kind() == SyntaxKind::REGISTER && t.kind() == SyntaxKind::REGISTER => {
            if let Some(lookup) = lookup {
                let actual = a.as_token().unwrap().text();
                let template = t.as_token().unwrap().text();

                lookup.get_size(actual) == DOC_REGISTERS.get_size(template)
                    || (lookup.is_sp(actual) && DOC_REGISTERS.is_sp(template))
                    || (template.starts_with("<R>")
                        && lookup
                            .get_kind(actual)
                            .contains(RegisterKind::GENERAL_PURPOSE))
            } else {
                false
            }
        }
        (a, t) if a.kind() == SyntaxKind::NUMBER && t.kind() == SyntaxKind::TOKEN => {
            // If we have a number and we are expecting a token, it could still
            // be a match if the token is an immediate
            t.as_token()
                // TODO: Could do more validation here
                .map(|t| t.text().starts_with('#'))
                .unwrap_or(false)
        }
        (a, t) if a.kind() == t.kind() => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests;
