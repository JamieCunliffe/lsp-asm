mod aarch64;
mod asm;
mod incomplete;
mod objdump;
mod x86_64;

#[macro_export]
macro_rules! assert_listing(
    ($src:expr, $expect:expr) => (
        assert_listing!($src, $expect, Default::default());

        // Run multiple times with different architectures as the default to ensure
        // that the detection is working correctly.
        for arch in base::Architecture::iter() {
            assert_listing!($src, $expect, *arch);
        }
    );
    ($src:expr, $expect:expr, $arch:expr) => (
        let config = $crate::config::LSPConfig {
            architecture: $arch,
            ..Default::default()
        };
        let parser = $crate::asm::parser::Parser::in_memory($src, &config);
        let nodes = parser.tree();
        pretty_assertions::assert_eq!(format!("{:#?}", nodes), $expect);
    )
);
