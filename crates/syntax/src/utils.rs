#[inline]
pub fn is_token_include(token: &str) -> bool {
    token.eq_ignore_ascii_case(".include")
        || token.eq_ignore_ascii_case("include")
        || token.eq_ignore_ascii_case("get")
        || token.eq_ignore_ascii_case("#include")
}
