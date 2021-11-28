use crate::ast::SyntaxToken;

#[inline]
pub fn is_token_include(token: &SyntaxToken) -> bool {
    token.text().eq_ignore_ascii_case(".include")
        || token.text().eq_ignore_ascii_case("include")
        || token.text().eq_ignore_ascii_case("get")
        || token.text().eq_ignore_ascii_case("#include")
}
