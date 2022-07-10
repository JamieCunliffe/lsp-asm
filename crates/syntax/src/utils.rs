use crate::ast::{SyntaxKind, SyntaxToken};

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
