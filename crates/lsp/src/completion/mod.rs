use std::sync::Arc;

use base::register::Registers;
use documentation::DocumentationMap;
use itertools::Itertools;
use rowan::NodeOrToken;
use syntax::alias::Alias;
use syntax::ast::{find_kind_index, find_parent, SyntaxElement, SyntaxKind, SyntaxToken};

use crate::asm::parser::Parser;
use crate::asm::registers::registers_for_architecture;
use crate::documentation::{find_potential_instruction_templates, parse_template};
use crate::types::{CompletionItem, DocumentPosition};

mod label;
mod mnemonic;
mod registers;
mod value_list;

pub fn handle_completion(
    parser: &Parser,
    position: &DocumentPosition,
    docs: Arc<DocumentationMap>,
) -> Option<Vec<CompletionItem>> {
    let location = parser.position().point_for_position(position)?;
    let token = parser.tree().token_at_offset(location).left_biased()?;
    let registers = registers_for_architecture(parser.architecture());

    let mut items = match token.kind() {
        SyntaxKind::MNEMONIC => mnemonic::handle_mnemonic(docs),
        SyntaxKind::WHITESPACE
            if matches!(
                token.parent()?.kind(),
                SyntaxKind::ROOT | SyntaxKind::LABEL | SyntaxKind::LOCAL_LABEL
            ) =>
        {
            mnemonic::handle_mnemonic(docs)
        }
        _ => {
            let completion_kinds = find_documentation_token_for_instruction(
                docs.clone(),
                &token,
                registers,
                parser.alias(),
            )?;

            let root = parser.tree();
            completion_kinds
                .iter()
                .flat_map(|token| match (token.kind(), token.text()) {
                    (_, "<label>") => label::complete_label(&root),
                    (SyntaxKind::REGISTER, _) => registers::complete_registers(
                        token.text(),
                        registers,
                        parser.architecture(),
                        parser.alias(),
                    ),
                    (SyntaxKind::TOKEN, ident)
                        if value_list::ident_can_complete(
                            ident.trim_start_matches('#'),
                            token,
                            docs.clone(),
                        ) =>
                    {
                        value_list::complete_ident(
                            ident.trim_start_matches('#'),
                            token,
                            docs.clone(),
                        )
                    }
                    _ => Default::default(),
                })
                .collect_vec()
        }
    };
    items.sort();
    items.dedup();
    Some(items)
}

fn find_documentation_token_for_instruction(
    docs: Arc<DocumentationMap>,
    token: &SyntaxToken,
    registers: &dyn Registers,
    alias: &Alias,
) -> Option<Vec<SyntaxToken>> {
    let instruction = find_parent(token, SyntaxKind::INSTRUCTION)?.clone_for_update();

    let token = if !is_real_token(&NodeOrToken::from(token.clone())) {
        std::iter::successors(token.prev_token(), |t| t.prev_token()).find(|t| {
            matches!(t.kind(), SyntaxKind::MNEMONIC) || is_real_token(&NodeOrToken::from(t.clone()))
        })?
    } else {
        token.clone()
    };

    let index = instruction
        .descendants_with_tokens()
        .filter(is_real_token)
        .enumerate()
        .find(|(_, t)| t.as_token().map(|t| t == &token).unwrap_or(false))
        .map(|(i, t)| {
            if matches!(t.kind(), SyntaxKind::COMMA) {
                i + 1
            } else {
                i
            }
        })
        .or_else(|| matches!(token.kind(), SyntaxKind::MNEMONIC).then(|| 0))?;

    let op = find_kind_index(&instruction, 0, SyntaxKind::MNEMONIC)?.into_token()?;
    let instructions = docs.get(&op.text().to_lowercase())?;

    // Remove the current token as this would interfere with the potential matches, for instance,
    // if x was typed this would be in the tree as a `SyntaxKind::TOKEN` and the match could be
    // expecting a `SyntaxKind::REGISTER` so it would discount it.
    if let Some(token) = instruction
        .token_at_offset(token.text_range().start())
        .right_biased()
    {
        if !matches!(token.kind(), SyntaxKind::COMMA) {
            token.detach();
        }
    }

    let templates =
        find_potential_instruction_templates(&instruction, instructions, registers, alias);

    let mut tokens = templates
        .iter()
        .flat_map(|template| template.asm.iter().map(parse_template))
        .filter_map(|asm| {
            asm.descendants_with_tokens()
                .filter(is_real_token)
                .filter_map(|d| d.into_token())
                .nth(index)
        })
        .collect_vec();
    tokens.sort_by(|a, b| a.text().cmp(b.text()));
    tokens.dedup_by(|a, b| a.text() == b.text());

    Some(tokens)
}

fn is_real_token(t: &SyntaxElement) -> bool {
    matches!(
        t.kind(),
        SyntaxKind::REGISTER
            | SyntaxKind::NUMBER
            | SyntaxKind::FLOAT
            | SyntaxKind::TOKEN
            | SyntaxKind::COMMA
    )
}
