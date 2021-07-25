use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use lsp_types::Url;
use once_cell::sync::OnceCell;

use super::ast::{self, SyntaxKind, SyntaxNode};
use crate::types::{DocumentLocation, DocumentPosition, DocumentRange, LineNumber};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct DebugMap {
    map: HashMap<u32, FileInfo>,
}

#[derive(Debug, Clone, PartialEq)]
struct FileInfo {
    name: String,
    contents: OnceCell<Vec<String>>,
}

impl DebugMap {
    pub(super) fn new(tree: &SyntaxNode) -> DebugMap {
        let map = tree
            .descendants()
            .filter(|d| {
                matches!(d.kind(), SyntaxKind::DIRECTIVE)
                    && ast::find_kind_index(d, 0, SyntaxKind::MNEMONIC)
                        .map(|n| n.as_token().map(|t| t.text() == ".file"))
                        .flatten()
                        .unwrap_or(false)
            })
            .filter_map(|n| {
                let id = ast::find_kind_index(&n, 0, SyntaxKind::NUMBER)?;
                let file = ast::find_kind_index(&n, 0, SyntaxKind::STRING)?;
                let name = file.as_token()?.text().trim_matches('"').to_string();

                let file = FileInfo {
                    name,
                    contents: Default::default(),
                };
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

    pub fn has_debug_map(&self) -> bool {
        !self.map.is_empty()
    }

    pub fn get_contents(&self, location: (u32, LineNumber)) -> Option<&String> {
        let (file, line) = location;

        self.map
            .get(&file)
            .map(|f| {
                f.contents
                    .get_or_init(|| {
                        Self::load_file(&f.name)
                            .map(|c| c.split('\n').map(String::from).collect())
                            .unwrap_or_default()
                    })
                    .get((line - 1) as usize)
            })
            .flatten()
    }

    pub fn get_location(&self, node: &SyntaxNode) -> Option<(u32, LineNumber)> {
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
        self.map.get(&id).map(|f| &f.name)
    }

    fn load_file(filename: &String) -> Option<String> {
        let mut file = File::open(filename).ok()?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        Some(contents)
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
        assert_eq!(
            values,
            vec![(
                &2,
                &FileInfo {
                    name: String::from("filename"),
                    contents: Default::default(),
                }
            )]
        );

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
