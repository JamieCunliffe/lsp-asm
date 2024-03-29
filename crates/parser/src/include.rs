use rowan::GreenNode;
use syntax::ast::SyntaxKind;
use syntax::utils::is_token_include;

use crate::config::ParserConfig;
use crate::{LoadFileFn, ParsedInclude};

pub(super) fn is_include(node: &GreenNode) -> bool {
    node.children()
        .find(|n| {
            (n.kind() == SyntaxKind::MNEMONIC.into())
                .then(|| n.as_token().map(|t| is_token_include(t.text())))
                .flatten()
                .unwrap_or(false)
        })
        .and_then(|t| t.as_token().map(|t| is_token_include(t.text())))
        .unwrap_or(false)
}

pub(super) fn handle_include(
    node: &GreenNode,
    config: &ParserConfig,
    from: Option<&str>,
    load: LoadFileFn,
) -> Option<ParsedInclude> {
    let filename = node
        .children()
        .find(|n| n.kind() == SyntaxKind::STRING.into())
        .and_then(|t| t.into_token())
        .map(|t| t.text().trim_matches('"').to_string())?;

    load(config, from.unwrap_or_default(), filename.as_str())
}
