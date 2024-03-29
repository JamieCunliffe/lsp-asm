use itertools::Itertools;
use syntax::ast::{find_parent, SyntaxKind, SyntaxToken};

use crate::types::{CompletionItem, CompletionKind};

use super::CompletionContext;

pub(super) fn complete_label(context: &CompletionContext) -> Vec<CompletionItem> {
    let token = |t: SyntaxToken| {
        (
            t.text().trim_end_matches(':').to_string(),
            crate::asm::hovers::label_definition_comment(context.parser, &t),
        )
    };
    let mut completions = find_parent(&context.token, SyntaxKind::LABEL)
        .map(|label| {
            label
                .descendants_with_tokens()
                .filter(|t| {
                    matches!(t.kind(), SyntaxKind::LABEL)
                        && matches!(t.parent().map(|p| p.kind()), Some(SyntaxKind::LOCAL_LABEL))
                })
                .filter_map(|t| t.into_token().map(token))
                .map(|(label, doc)| to_completion(label, doc))
                .collect_vec()
        })
        .unwrap_or_default();

    if let Some(labels) = find_parent(&context.token, SyntaxKind::ROOT).map(|n| {
        n.descendants_with_tokens()
            .filter(|t| {
                matches!(t.kind(), SyntaxKind::LABEL)
                    && matches!(t.parent().map(|p| p.kind()), Some(SyntaxKind::LABEL))
            })
            .filter_map(|t| t.into_token().map(token))
            .map(|(label, doc)| to_completion(label, doc))
    }) {
        completions.extend(labels);
    };

    completions
}

fn to_completion(l: String, doc: Option<String>) -> CompletionItem {
    CompletionItem {
        text: l,
        details: "".into(),
        documentation: doc,
        kind: CompletionKind::Label,
    }
}

#[cfg(test)]
mod tests {
    use crate::asm::parser::Parser;

    use super::*;

    use pretty_assertions::assert_eq;
    use syntax::ast::find_kind_index;

    #[test]
    fn completion_labels() {
        let data = r#"label1:
main:
another_label:
b "#;
        let parser = Parser::in_memory(data, &Default::default());
        let context = CompletionContext::new(
            &parser,
            find_kind_index(&parser.tree(), 1, SyntaxKind::LABEL)
                .unwrap()
                .into_token()
                .unwrap(),
            Default::default(),
        );

        let results = complete_label(&context);
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
