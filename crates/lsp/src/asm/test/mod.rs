mod asm;
mod objdump;

#[macro_export]
macro_rules! assert_listing(
    ($src:expr, $expect:expr) => (
        assert_listing!($src, $expect, Default::default());
    );
    ($src:expr, $expect:expr, $arch:expr) => (
        let config = crate::config::LSPConfig {
            architecture: $arch,
            ..Default::default()
        };
        let parser = crate::asm::parser::Parser::from($src, &config);
        let nodes = parser.tree();
        pretty_assertions::assert_eq!(format!("{:#?}", nodes), $expect);
    )
);
