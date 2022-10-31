use crate::handler::context::Context;

use super::ast::{LabelToken, NumericToken};
use super::definition::get_definition_token;
use super::parser::Parser;
use arch::registers::registers_for_architecture;
use base::Architecture;
use itertools::Itertools;
use rowan::NodeOrToken;
use std::iter;
use std::sync::Arc;
use syntax::alias::Alias;
use syntax::ast::{self, SyntaxKind, SyntaxToken};

pub fn get_numeric_hover(value: &NumericToken) -> Option<Vec<String>> {
    let value = value.value();
    Some(vec![
        "# Number".to_string(),
        format!("Decimal: {value}"),
        format!("Hex: {value:#X}"),
    ])
}

pub fn get_label_hover(label: &LabelToken) -> Option<Vec<String>> {
    let mut symbols = Vec::new();

    if let Some((sym, lang)) = label.demangle() {
        symbols.push(String::from("# Demangled Symbol"));
        symbols.push(format!("**{lang}**: `{sym}`"));
    }

    Some(symbols)
}

pub fn get_hover_mnemonic(
    token: &SyntaxToken,
    arch: &Architecture,
    alias: &Alias,
) -> Option<Vec<String>> {
    let instruction = ast::find_parent(token, SyntaxKind::INSTRUCTION)?;

    let docs = documentation::load_documentation(arch).ok()?;
    let instructions = docs.get(token.text())?;

    let template = documentation::templates::find_correct_instruction_template(
        &instruction,
        instructions,
        registers_for_architecture(arch),
        alias,
        *arch,
    );

    if let Some(template) = template {
        let instruction =
            documentation::templates::instruction_from_template(instructions, template)?;

        Some(vec![format!("{instruction}")])
    } else {
        // Couldn't resolve which instruction we are on so print them all.
        Some(
            instructions
                .iter()
                .map(|i| format!("{i}"))
                .interleave_shortest(iter::repeat(String::from("---")))
                .collect(),
        )
    }
}

pub fn get_alias_hover(token: &SyntaxToken, alias: &Alias) -> Option<Vec<String>> {
    let register = alias.get_register_for_alias(token.text())?;
    Some(vec![format!(
        "`{}` is an alias to register `{register}`",
        token.text(),
    )])
}

pub fn get_constant_hover(token: &SyntaxToken, alias: &Alias) -> Option<Vec<String>> {
    let value = alias.get_constant_for_token(token.text())?;
    Some(vec![format!(
        "`{}` is defined as `{}`",
        token.text(),
        value.trim()
    )])
}

pub fn get_token_hover(
    context: Arc<Context>,
    parser: &Parser,
    token: SyntaxToken,
) -> Option<Vec<String>> {
    let doc_strings =
        get_definition_token(context, parser, &token, label_definition_comment).ok()?;

    Some(doc_strings)
}

pub(crate) fn label_definition_comment(
    parser: &Parser,
    definition: &SyntaxToken,
) -> Option<String> {
    let first = match definition.parent()?.prev_sibling_or_token() {
        Some(NodeOrToken::Node(n)) => n.last_token().map(NodeOrToken::Token),
        x => x,
    };

    let mut comments = iter::successors(first, |t| t.prev_sibling_or_token())
        .take_while(|t| matches!(t.kind(), SyntaxKind::WHITESPACE | SyntaxKind::COMMENT))
        .collect_vec();

    comments.reverse();

    let comment = comments
        .iter()
        .filter_map(|s| {
            s.as_token().map(|t| {
                t.text()
                    .trim_start_matches(parser.comment_start())
                    .trim_start()
            })
        })
        .join("\n")
        .trim()
        .to_string();

    (!comment.is_empty()).then_some(comment)
}
