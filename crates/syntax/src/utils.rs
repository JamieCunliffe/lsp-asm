use crate::ast::{SyntaxKind, SyntaxNode, SyntaxToken};

#[inline]
pub fn is_token_include(token: &str) -> bool {
    token.eq_ignore_ascii_case(".include")
        || token.eq_ignore_ascii_case("include")
        || token.eq_ignore_ascii_case("get")
        || token.eq_ignore_ascii_case("#include")
}

#[inline]
pub fn token_is_local_label(token: &SyntaxToken) -> bool {
    matches!(token.kind(), SyntaxKind::TOKEN | SyntaxKind::LABEL) && token.text().starts_with('.')
}

pub fn find_token_containing(root: &SyntaxNode, text: &str) -> Option<SyntaxToken> {
    root.descendants_with_tokens().find_map(|elem| match elem {
        rowan::NodeOrToken::Node(_) => None,
        rowan::NodeOrToken::Token(token) => token.text().contains(text).then_some(token),
    })
}
