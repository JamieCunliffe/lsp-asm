use base::register::{RegisterSize, Registers};
use base::Architecture;
use itertools::Itertools;
use parser::config::ParserConfig;
use parser::ParsedData;
use syntax::alias::Alias;
use syntax::ast::{find_parent_elem, SyntaxElement, SyntaxKind, SyntaxNode};

use crate::registers::DOC_REGISTERS;
use crate::{Instruction, InstructionTemplate};

fn get_config(architecture: Architecture) -> ParserConfig {
    ParserConfig {
        comment_start: String::from("\u{00A0}"),
        architecture,
        file_type: Default::default(),
        registers: Some(&crate::registers::DOCUMENTATION_REGISTERS),
    }
}

/// Finds the instruction template that matches the given node, this will
/// require an exact match to be found otherwise None is returned
pub fn find_correct_instruction_template<'a>(
    node: &SyntaxNode,
    instructions: &'a [Instruction],
    lookup: &dyn Registers,
    alias: &Alias,
    arch: Architecture,
) -> Option<&'a InstructionTemplate> {
    instructions
        .iter()
        .flat_map(|x| &x.asm_template)
        .find(|template| {
            template
                .asm
                .iter()
                .any(|template| check_template(template, node, true, lookup, alias, arch))
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
    arch: Architecture,
) -> Vec<&'a InstructionTemplate> {
    instructions
        .iter()
        .flat_map(|x| &x.asm_template)
        .filter(|template| is_potential_instruction_template(node, template, lookup, alias, arch))
        .collect_vec()
}

pub fn is_potential_instruction_template(
    node: &SyntaxNode,
    template: &InstructionTemplate,
    lookup: &dyn Registers,
    alias: &Alias,
    arch: Architecture,
) -> bool {
    template
        .asm
        .iter()
        .any(|template| check_template(template, node, false, lookup, alias, arch))
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

pub fn parse_template<T>(template: T, arch: Architecture) -> SyntaxNode
where
    T: AsRef<str>,
{
    let ParsedData { root, .. } =
        parser::parse_asm(template.as_ref(), &get_config(arch), None, |_, _, _| None);
    SyntaxNode::new_root(root)
}

fn check_template(
    template: &str,
    node: &SyntaxNode,
    exact: bool,
    lookup: &dyn Registers,
    alias: &Alias,
    arch: Architecture,
) -> bool {
    let parsed_template = parse_template(template, arch);

    let parsed_template = parsed_template.first_child().unwrap();
    let filtered = node
        .descendants_with_tokens()
        .filter(|c| {
            !matches!(
                c.kind(),
                SyntaxKind::WHITESPACE
                    | SyntaxKind::METADATA
                    | SyntaxKind::OBJDUMP_OFFSET
                    | SyntaxKind::COMMENT
                    | SyntaxKind::RELOCATION
            ) && find_parent_elem(c, SyntaxKind::METADATA).is_none()
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
            true
        }
        (a, t) if matches!(a.kind(), SyntaxKind::CONSTANT) && t.kind() == SyntaxKind::TOKEN => true,
        (a, t) if a.kind() == t.kind() => true,
        _ => false,
    }
}

fn register_match(actual: &str, template: &str, lookup: &dyn Registers) -> bool {
    let doc_size = DOC_REGISTERS.get_size(template);
    let size = lookup.get_size(actual) == doc_size || doc_size == RegisterSize::Any;

    size && DOC_REGISTERS
        .get_kind(template)
        .contains(lookup.get_kind(actual))
}
