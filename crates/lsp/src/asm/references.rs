use std::iter;

use rowan::TextRange;
use syntax::ast::{find_parent, SyntaxKind, SyntaxToken};
use syntax::utils::token_is_local_label;

use super::parser::Parser;

pub fn get_search_range(parser: &Parser, token: &SyntaxToken, limit: Option<u32>) -> TextRange {
    if token_is_local_label(token) {
        token
            .text()
            .starts_with('.')
            .then(|| find_parent(token, SyntaxKind::LABEL).map(|label| label.text_range()))
            .flatten()
            .unwrap_or_else(|| parser.text_range())
    } else if let Some(limit) = limit {
        let position = parser.position().get_position(token).unwrap();
        parser.position().make_range_for_lines(
            position.line.saturating_sub(limit),
            position.line.saturating_add(limit),
        )
    } else {
        parser.text_range()
    }
}

pub fn find_references<'a>(
    parser: &'a Parser,
    token: &'a SyntaxToken,
    range: TextRange,
    include_decl: bool,
) -> Box<dyn Iterator<Item = SyntaxToken> + 'a> {
    if matches!(token.kind(), SyntaxKind::NUMBER | SyntaxKind::MNEMONIC) {
        return Box::new(iter::empty::<SyntaxToken>());
    }

    let references = parser
        .tokens_in_range(range)
        .filter(move |t| parser.token_text_equal(token, t));

    let label_fn =
        |t: &SyntaxToken| !(t.kind() == SyntaxKind::LABEL || t.kind() == SyntaxKind::LOCAL_LABEL);
    let const_fn = |t: &SyntaxToken| find_parent(t, SyntaxKind::CONST_DEF).is_none();
    let alias_fn = |t: &SyntaxToken| find_parent(t, SyntaxKind::ALIAS).is_none();

    match token.kind() {
        _ if include_decl => Box::new(references),
        SyntaxKind::LABEL | SyntaxKind::TOKEN => Box::new(references.filter(label_fn)),
        SyntaxKind::CONSTANT | SyntaxKind::NAME => Box::new(references.filter(const_fn)),
        SyntaxKind::REGISTER_ALIAS | SyntaxKind::REGISTER => Box::new(references.filter(alias_fn)),
        _ => Box::new(references),
    }
}

pub fn find_references_alias_exact<'a>(
    parser: &'a Parser,
    token: &'a SyntaxToken,
    range: TextRange,
) -> Box<dyn Iterator<Item = SyntaxToken> + 'a> {
    Box::new(
        parser
            .tokens_in_range(range)
            .filter(move |t| matches!(t.kind(), SyntaxKind::REGISTER_ALIAS))
            .filter(move |t| t.text() == token.text()),
    )
}
