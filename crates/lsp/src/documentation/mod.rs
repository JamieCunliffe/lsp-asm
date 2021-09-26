use itertools::Itertools;

use base::register::{RegisterSize, Registers};
use base::Architecture;
use documentation::registers::DOC_REGISTERS;
use documentation::{Instruction, InstructionTemplate};
use lazy_static::lazy_static;
use parser::config::ParserConfig;
use parser::ParsedData;
use syntax::alias::Alias;
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
    lookup: &dyn Registers,
    alias: &Alias,
) -> Option<&'a InstructionTemplate> {
    instructions
        .iter()
        .flat_map(|x| &x.asm_template)
        .find(|template| {
            template
                .asm
                .iter()
                .any(|template| check_template(template, node, true, lookup, alias))
        })
}

/// Finds the instruction templates that match the given node, this will return
/// templates that are a partial match i.e. if the template is longer than the
/// input but up to that point it matches, then it will be included.
pub fn find_potential_instruction_templates<'a>(
    node: &SyntaxNode,
    instructions: &'a [Instruction],
    lookup: &dyn Registers,
    alias: &Alias,
) -> Vec<&'a InstructionTemplate> {
    instructions
        .iter()
        .flat_map(|x| &x.asm_template)
        .filter(|template| is_potential_instruction_template(node, template, lookup, alias))
        .collect_vec()
}

pub fn is_potential_instruction_template(
    node: &SyntaxNode,
    template: &InstructionTemplate,
    lookup: &dyn Registers,
    alias: &Alias,
) -> bool {
    template
        .asm
        .iter()
        .any(|template| check_template(template, node, false, lookup, alias))
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

pub(crate) fn parse_template<T>(template: T) -> SyntaxNode
where
    T: AsRef<str>,
{
    let ParsedData { root, .. } = parser::parse_asm(template.as_ref(), &TEMPLATE_CONFIG);
    SyntaxNode::new_root(root)
}

fn check_template(
    template: &str,
    node: &SyntaxNode,
    exact: bool,
    lookup: &dyn Registers,
    alias: &Alias,
) -> bool {
    let parsed_template = parse_template(template);

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
            .all(|x| node_or_token_match(x, lookup, alias))
}

fn node_or_token_match(
    elements: (&SyntaxElement, &SyntaxElement),
    lookup: &dyn Registers,
    alias: &Alias,
) -> bool {
    match elements {
        (a, t) if a.kind() == SyntaxKind::REGISTER && t.kind() == SyntaxKind::REGISTER => {
            let actual = a.as_token().unwrap().text();
            let template = t.as_token().unwrap().text();
            register_match(actual, template, lookup)
        }
        (a, t) if a.kind() == SyntaxKind::REGISTER_ALIAS && t.kind() == SyntaxKind::REGISTER => {
            alias
                .get_register_for_alias(a.as_token().unwrap().text())
                .map(|actual| {
                    let template = t.as_token().unwrap().text();
                    register_match(actual, template, lookup)
                })
                .unwrap_or(false)
        }
        (a, t)
            if matches!(a.kind(), SyntaxKind::NUMBER | SyntaxKind::FLOAT)
                && t.kind() == SyntaxKind::TOKEN =>
        {
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

fn register_match(actual: &str, template: &str, lookup: &dyn Registers) -> bool {
    let doc_size = DOC_REGISTERS.get_size(template);
    let size = lookup.get_size(actual) == doc_size || doc_size == RegisterSize::Unknown;

    size && DOC_REGISTERS
        .get_kind(template)
        .contains(lookup.get_kind(actual))
}

#[cfg(test)]
mod tests;
