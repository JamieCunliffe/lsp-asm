use syntax::ast::{SyntaxKind, SyntaxToken};

use crate::types::{CompletionItem, CompletionKind};

fn get_text(token: &SyntaxToken) -> Option<&str> {
    if matches!(token.kind(), SyntaxKind::COMMENT) {
        token.text().trim_end().split_ascii_whitespace().last()
    } else {
        Some(token.text())
    }
}

pub(super) fn handle(token: SyntaxToken) -> Option<Vec<CompletionItem>> {
    let prev = token.prev_token();
    let prev = prev.as_ref();

    let token = if token.text().contains("lsp-asm-") {
        get_text(&token)
    } else {
        get_text(prev?)
    }?;

    if token.starts_with("lsp-asm-architecture") {
        Some(
            base::Architecture::iter()
                .map(|arch| CompletionItem {
                    text: arch.to_string(),
                    details: "".into(),
                    documentation: None,
                    kind: CompletionKind::Text,
                })
                .collect(),
        )
    } else {
        None
    }
}
