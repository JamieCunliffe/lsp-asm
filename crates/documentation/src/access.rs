use base::register::Registers;
use base::Architecture;
use syntax::alias::Alias;
use syntax::ast::{find_parent, SyntaxKind, SyntaxToken};

use crate::templates::find_correct_instruction_template;
use crate::{DocumentationMap, OperandAccessType};

pub fn access_type(
    token: &SyntaxToken,
    docs: &DocumentationMap,
    registers: &dyn Registers,
    alias: &Alias,
    arch: Architecture,
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
        find_correct_instruction_template(&instruction, instructions, registers, alias, arch)?;

    template.access_map.get(index).cloned()
}
