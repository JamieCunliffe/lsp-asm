use syntax::ast::SyntaxNode;

#[macro_export]
macro_rules! format_test {
    ($input:literal => $expected:literal, $opts:expr, $pass:ident) => {
        let (input, _) = $crate::test_util::parse_asm($input);
        let (expected, _) = $crate::test_util::parse_asm($expected);

        let actual = $pass(input.clone_for_update(), $opts);
        pretty_assertions::assert_eq!(format!("{expected:#?}"), format!("{actual:#?}"));
    };
}

pub fn parse_asm(data: &str) -> (SyntaxNode, syntax::alias::Alias) {
    use arch::register_names::AARCH64_REGISTERS;
    use parser::{config::ParserConfig, ParsedData, ParsedInclude};

    let config = ParserConfig {
        comment_start: String::from("//"),
        architecture: base::Architecture::AArch64,
        file_type: base::FileType::Assembly,
        registers: Some(&AARCH64_REGISTERS),
    };
    let load_file = |_current_config: &ParserConfig,
                     _current_file: &str,
                     _filename: &str|
     -> Option<ParsedInclude> { None };

    let ParsedData { root, alias, .. } = parser::parse_asm(data, &config, None, load_file);
    (SyntaxNode::new_root(root), alias)
}
