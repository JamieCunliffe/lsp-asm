use itertools::Itertools;
use syntax::ast::{SyntaxKind, SyntaxNode};

use crate::types::{CompletionItem, CompletionKind};

pub(super) fn complete_label(root: &SyntaxNode) -> Vec<CompletionItem> {
    root.descendants_with_tokens()
        .filter(|t| matches!(t.kind(), SyntaxKind::LABEL | SyntaxKind::LOCAL_LABEL))
        .filter_map(|t| {
            t.as_token()
                .map(|t| t.text().trim_end_matches(':').to_string())
        })
        .map(|l| CompletionItem {
            text: l,
            details: "".into(),
            documentation: None,
            kind: CompletionKind::Label,
        })
        .collect_vec()
}

#[cfg(test)]
mod tests {
    use crate::asm::parser::Parser;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn completion_labels() {
        let data = r#"label1:
main:
another_label:
b "#;
        let parser = Parser::from(data, &Default::default());
        let root = parser.tree();
        let results = complete_label(&root);
        let expected = vec!["label1", "main", "another_label"]
            .into_iter()
            .map(|l| CompletionItem {
                text: l.to_string(),
                details: "".into(),
                documentation: None,
                kind: CompletionKind::Label,
            })
            .collect_vec();

        assert_eq!(results, expected);
    }
}
