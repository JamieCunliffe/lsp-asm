use std::collections::HashMap;

use lsp_types::Url;

use crate::{
    asm::ast::find_kind_index,
    types::{DocumentLocation, DocumentPosition, DocumentRange, LineNumber},
};

use super::ast::{self, SyntaxKind, SyntaxNode};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct DebugMap {
    map: HashMap<u32, String>,
}

impl DebugMap {
    pub(super) fn new(tree: &SyntaxNode) -> DebugMap {
        let map = tree
            .descendants()
            .filter(|d| {
                matches!(d.kind(), SyntaxKind::DIRECTIVE)
                    && find_kind_index(d, 0, SyntaxKind::MNEMONIC)
                        .map(|n| n.as_token().map(|t| t.text() == ".file"))
                        .flatten()
                        .unwrap_or(false)
            })
            .filter_map(|n| {
                let id = ast::find_kind_index(&n, 0, SyntaxKind::NUMBER)?;
                let file = ast::find_kind_index(&n, 0, SyntaxKind::STRING)?;
                let file = file.as_token()?.text().trim_matches('"').to_string();

                Some((id.as_token()?.text().parse::<u32>().ok()?, file))
            })
            .collect();

        DebugMap { map }
    }

    pub fn get_file_location(&self, node: &SyntaxNode) -> Option<DocumentLocation> {
        let (file_id, line) = self.get_location(node)?;
        let line = line - 1;
        let file = self.get_filename(file_id)?;
        Some(DocumentLocation {
            uri: Url::parse(format!("file://{}", file).as_str()).ok()?,
            range: DocumentRange {
                start: DocumentPosition { line, column: 0 },
                end: DocumentPosition { line, column: 0 },
            },
        })
    }

    fn get_location(&self, node: &SyntaxNode) -> Option<(u32, LineNumber)> {
        if matches!(node.kind(), SyntaxKind::DIRECTIVE) {
            ast::find_kind_index(&node, 0, SyntaxKind::MNEMONIC)
                .filter(|t| t.as_token().map(|t| t.text() == ".loc").unwrap_or(false))
                .and_then(|_| {
                    let file_id = ast::find_kind_index(&node, 0, SyntaxKind::NUMBER)?;
                    let file_id = file_id.as_token()?.text().parse::<u32>().ok()?;
                    let line = ast::find_kind_index(&node, 1, SyntaxKind::NUMBER)?;
                    let line = line.as_token()?.text().parse::<LineNumber>().ok()?;
                    Some((file_id, line))
                })
        } else {
            None
        }
    }

    fn get_filename(&self, id: u32) -> Option<&String> {
        self.map.get(&id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::asm::parser::Parser;

    use pretty_assertions::assert_eq;

    #[test]
    fn debug_map() {
        let data = r#"Lfunc_begin0:
	.file	2 "filename"
	.loc	2 2132 0
	.cfi_startproc"#;
        let tree = Parser::from(data, &Default::default());
        let map = DebugMap::new(tree.tree());

        let values = map.map.iter().clone().collect::<Vec<_>>();
        assert_eq!(values, vec![(&2, &String::from("filename"))]);

        assert_eq!(
            map.get_file_location(
                ast::find_kind_index(tree.tree(), 1, SyntaxKind::DIRECTIVE)
                    .unwrap()
                    .as_node()
                    .unwrap()
            ),
            Some(DocumentLocation {
                uri: Url::parse("file://filename").unwrap(),
                range: DocumentRange {
                    start: DocumentPosition {
                        line: 2131,
                        column: 0
                    },
                    end: DocumentPosition {
                        line: 2131,
                        column: 0
                    },
                },
            })
        );
    }
}
