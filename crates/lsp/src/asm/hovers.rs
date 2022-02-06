use super::ast::{LabelToken, NumericToken};
use super::definition::get_definition_token;
use super::parser::Parser;
use super::registers::registers_for_architecture;
use base::Architecture;
use itertools::Itertools;
use rowan::NodeOrToken;
use std::iter;
use syntax::alias::Alias;
use syntax::ast::{self, SyntaxKind, SyntaxToken};

pub fn get_numeric_hover(value: &NumericToken) -> Option<Vec<String>> {
    let value = value.value();
    Some(vec![
        "# Number".to_string(),
        format!("Decimal: {}", value),
        format!("Hex: {:#X}", value),
    ])
}

pub fn get_label_hover(label: &LabelToken) -> Option<Vec<String>> {
    let mut symbols = Vec::new();

    if let Some((sym, lang)) = label.demangle() {
        symbols.push(String::from("# Demangled Symbol"));
        symbols.push(format!("**{}**: `{}`", lang, sym));
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

    let template = crate::documentation::find_correct_instruction_template(
        &instruction,
        instructions,
        registers_for_architecture(arch),
        alias,
    );

    if let Some(template) = template {
        let instruction = crate::documentation::instruction_from_template(instructions, template)?;

        Some(vec![format!("{}", instruction)])
    } else {
        // Couldn't resolve which instruction we are on so print them all.
        Some(
            instructions
                .iter()
                .map(|i| format!("{}", i))
                .interleave_shortest(iter::repeat(String::from("---")))
                .collect(),
        )
    }
}

pub fn get_alias_hover(token: &SyntaxToken, alias: &Alias) -> Option<Vec<String>> {
    let register = alias.get_register_for_alias(token.text())?;
    Some(vec![format!(
        "`{}` is an alias to register `{}`",
        token.text(),
        register
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

pub fn get_token_hover(parser: &Parser, token: SyntaxToken) -> Option<Vec<String>> {
    let definitions = get_definition_token(parser, &token).ok()?;
    let doc_strings = definitions
        .filter_map(label_definition_comment)
        .collect_vec();

    Some(doc_strings)
}

pub(crate) fn label_definition_comment(definition: (SyntaxToken, &Parser)) -> Option<String> {
    let (definition, parser) = definition;
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

    (!comment.is_empty()).then(|| comment)
}
