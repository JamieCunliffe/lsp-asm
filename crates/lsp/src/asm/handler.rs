#![allow(deprecated)]
#![allow(unused)]
use std::str::FromStr;

use lsp_server::ResponseError;
use lsp_types::{
    DocumentHighlight, DocumentHighlightKind, DocumentSymbol, DocumentSymbolResponse,
    GotoDefinitionResponse, HoverContents, Location, MarkupContent, Position, Range, SemanticToken,
    SemanticTokens, SemanticTokensResult, SymbolKind, Url,
};
use rowan::TextRange;

use crate::asm::ast::RegisterToken;
use crate::asm::combinators;
use crate::config::LSPConfig;
use crate::handler::error::{lsp_error_map, ErrorCode};
use crate::handler::semantic::{self, semantic_delta_transform};
use crate::handler::LanguageServerProtocol;
use crate::types::DocumentPosition;

use super::ast::{LabelToken, NumericToken};
use super::ast::{SyntaxKind, SyntaxNode, SyntaxToken};
use super::config::ParserConfig;

use super::parser::{Parser, PositionInfo};
use super::registers::{registers_for_architecture, RegisterKind, Registers};

pub struct AssemblyLanguageServerProtocol {
    parser: Parser,
    uri: Url,
    config: LSPConfig,
}

impl LanguageServerProtocol for AssemblyLanguageServerProtocol {
    fn update(&mut self, data: &str) {
        self.parser = Parser::from(data, &self.config);
    }

    fn goto_definition(
        &self,
        position: DocumentPosition,
    ) -> Result<lsp_types::GotoDefinitionResponse, lsp_server::ResponseError> {
        let token = self
            .parser
            .token_at_point(&position)
            .ok_or_else(|| lsp_error_map(ErrorCode::TokenNotFound))?;
        let position = self.parser.position();

        let res = match token.kind() {
            SyntaxKind::TOKEN => self
                .parser
                .tree()
                .descendants_with_tokens()
                .filter_map(|d| d.into_token())
                .filter(|token| token.kind() == SyntaxKind::LABEL)
                .filter(|label| {
                    self.parser
                        .token::<LabelToken>(label)
                        .map(|name| name.name() == token.text())
                        .unwrap_or(false)
                })
                .filter_map(|token| {
                    Some(lsp_types::Location::new(
                        self.uri.clone(),
                        position.range_for_token(&token)?.into(),
                    ))
                })
                .collect(),
            SyntaxKind::MNEMONIC if token.text() == ".loc" => vec![self
                .parser
                .debug_map()
                .get_file_location(&token.parent())
                .map(|l| l.into())
                .ok_or_else(|| lsp_error_map(ErrorCode::InvalidPosition))?],
            _ => Vec::new(),
        };

        Ok(lsp_types::GotoDefinitionResponse::Array(res))
    }

    fn find_references(
        &self,
        position: DocumentPosition,
        include_decl: bool,
    ) -> Result<Vec<Location>, lsp_server::ResponseError> {
        let token = self
            .parser
            .token_at_point(&position)
            .ok_or_else(|| lsp_error_map(ErrorCode::TokenNotFound))?;

        if !matches!(token.kind(), SyntaxKind::LABEL | SyntaxKind::TOKEN) {
            return Ok(Vec::new());
        }

        let references = find_references(&self.parser, &token);
        let position = self.parser.position();
        let locations = references
            .filter(|t| {
                include_decl
                    || !(t.kind() == SyntaxKind::LABEL || t.kind() == SyntaxKind::LOCAL_LABEL)
            })
            .filter_map(|token| {
                Some(lsp_types::Location::new(
                    self.uri.clone(),
                    position.range_for_token(&token)?.into(),
                ))
            })
            .collect();

        Ok(locations)
    }

    fn hover(
        &self,
        position: DocumentPosition,
    ) -> Result<Option<lsp_types::Hover>, lsp_server::ResponseError> {
        let token = self
            .parser
            .token_at_point(&position)
            .ok_or_else(|| lsp_error_map(ErrorCode::TokenNotFound))?;

        let hover = match token.kind() {
            SyntaxKind::NUMBER => get_numeric_hover(
                &self
                    .parser
                    .token(&token)
                    .ok_or_else(|| lsp_error_map(ErrorCode::CastFailed))?,
            ),
            SyntaxKind::LABEL => get_label_hover(
                &self
                    .parser
                    .token(&token)
                    .ok_or_else(|| lsp_error_map(ErrorCode::CastFailed))?,
            ),
            SyntaxKind::L_PAREN
            | SyntaxKind::R_PAREN
            | SyntaxKind::L_SQ
            | SyntaxKind::R_SQ
            | SyntaxKind::L_CURLY
            | SyntaxKind::R_CURLY
            | SyntaxKind::L_ANGLE
            | SyntaxKind::R_ANGLE
            | SyntaxKind::MNEMONIC
            | SyntaxKind::REGISTER
            | SyntaxKind::TOKEN
            | SyntaxKind::WHITESPACE
            | SyntaxKind::COMMA
            | SyntaxKind::OPERATOR
            | SyntaxKind::STRING
            | SyntaxKind::LOCAL_LABEL
            | SyntaxKind::COMMENT
            | SyntaxKind::INSTRUCTION
            | SyntaxKind::DIRECTIVE
            | SyntaxKind::BRACKETS
            | SyntaxKind::METADATA
            | SyntaxKind::ROOT => None,
        };

        debug!("hover: {:#?}", hover);

        Ok(hover.map(|mut hover| lsp_types::Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: lsp_types::MarkupKind::Markdown,
                value: hover.join("\n"),
            }),
            range: None,
        }))
    }

    fn document_highlight(
        &self,
        position: DocumentPosition,
    ) -> Result<Vec<lsp_types::DocumentHighlight>, lsp_server::ResponseError> {
        let token = self
            .parser
            .token_at_point(&position)
            .ok_or_else(|| lsp_error_map(ErrorCode::TokenNotFound))?;

        if matches!(token.kind(), SyntaxKind::NUMBER) {
            return Ok(Vec::new());
        }
        let position_cache = self.parser.position();
        let range = position_cache.make_range_for_lines(
            position.line.saturating_sub(200),
            position.line.saturating_add(200),
        );

        let references = find_references(&self.parser, &token);
        let locations = references
            .filter(|token| range.contains(token.text_range().start()))
            .filter_map(|token| {
                Some(lsp_types::DocumentHighlight {
                    range: position_cache.range_for_token(&token)?.into(),
                    kind: Some(DocumentHighlightKind::Text),
                })
            })
            .collect();

        Ok(locations)
    }

    fn get_semantic_tokens(
        &self,
        range: Option<Range>,
    ) -> Result<lsp_types::SemanticTokensResult, lsp_server::ResponseError> {
        let range = if let Some(range) = range {
            let start = self
                .parser
                .position()
                .point_for_position(&range.start.into())
                .ok_or_else(|| lsp_error_map(ErrorCode::InvalidPosition))?;
            let end = self
                .parser
                .position()
                .point_for_position(&range.end.into())
                .ok_or_else(|| lsp_error_map(ErrorCode::InvalidPosition))?;
            TextRange::new(start, end)
        } else {
            self.parser.tree().text_range()
        };

        let position = self.parser.position();
        let tokens = self.parser.tokens_in_range(&range);

        let tokens = tokens
            .iter()
            .filter_map(|token| {
                if let Some(index) = match token.kind() {
                    _ if crate::asm::ast::find_parent(&token, SyntaxKind::METADATA).is_some() => {
                        Some(crate::handler::semantic::METADATA_INDEX)
                    }
                    SyntaxKind::METADATA => Some(crate::handler::semantic::METADATA_INDEX),
                    SyntaxKind::MNEMONIC => match token.parent().kind() {
                        SyntaxKind::INSTRUCTION => Some(crate::handler::semantic::OPCODE_INDEX),
                        SyntaxKind::DIRECTIVE => Some(crate::handler::semantic::DIRECTIVE_INDEX),
                        _ => unreachable!("Parent should be instruction or directive"),
                    },
                    SyntaxKind::COMMENT => Some(crate::handler::semantic::COMMENT_INDEX),
                    SyntaxKind::NUMBER => Some(crate::handler::semantic::NUMERIC_INDEX),
                    SyntaxKind::STRING => Some(crate::handler::semantic::STRING_INDEX),
                    SyntaxKind::REGISTER => self
                        .parser
                        .token::<RegisterToken>(token)
                        .map(|register| {
                            let kind = register.register_kind();

                            if kind.contains(RegisterKind::GENERAL_PURPOSE) {
                                crate::handler::semantic::GP_REGISTER_INDEX
                            } else if kind.contains(RegisterKind::FLOATING_POINT) {
                                crate::handler::semantic::FP_REGISTER_INDEX
                            } else {
                                crate::handler::semantic::REGISTER_INDEX
                            }
                        })
                        .or(Some(crate::handler::semantic::REGISTER_INDEX)),
                    SyntaxKind::LABEL | SyntaxKind::LOCAL_LABEL => {
                        Some(crate::handler::semantic::LABEL_INDEX)
                    }
                    SyntaxKind::L_PAREN
                    | SyntaxKind::R_PAREN
                    | SyntaxKind::L_SQ
                    | SyntaxKind::R_SQ
                    | SyntaxKind::L_CURLY
                    | SyntaxKind::R_CURLY
                    | SyntaxKind::L_ANGLE
                    | SyntaxKind::R_ANGLE
                    | SyntaxKind::TOKEN
                    | SyntaxKind::WHITESPACE
                    | SyntaxKind::COMMA
                    | SyntaxKind::OPERATOR
                    | SyntaxKind::INSTRUCTION
                    | SyntaxKind::DIRECTIVE
                    | SyntaxKind::BRACKETS
                    | SyntaxKind::ROOT => None,
                } {
                    let pos = position.get_position(token)?;
                    Some(SemanticToken {
                        delta_line: pos.line,
                        delta_start: pos.column,
                        length: token.text_range().len().into(),
                        token_type: index,
                        token_modifiers_bitset: 0,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let tokens = semantic_delta_transform(&tokens);
        Ok(SemanticTokensResult::Tokens(SemanticTokens {
            data: tokens,
            result_id: None,
        }))
    }

    fn document_symbols(
        &self,
    ) -> Result<lsp_types::DocumentSymbolResponse, lsp_server::ResponseError> {
        Ok(DocumentSymbolResponse::Nested(
            self.parser
                .tree()
                .first_child()
                .map(|root| {
                    root.siblings_with_tokens(rowan::Direction::Next)
                        .filter_map(|node| {
                            node.as_node().and_then(|node| match node.kind() {
                                SyntaxKind::LABEL => node
                                    .descendants_with_tokens()
                                    .find(|n| {
                                        n.as_token().map(|n| n.kind()) == Some(SyntaxKind::LABEL)
                                    })
                                    .map(|t| t.into_token())
                                    .flatten()
                                    .map(|token| {
                                        node_to_document_symbol(
                                            self.parser.position(),
                                            &node,
                                            &token,
                                            Some(
                                                find_nodes(&node, SyntaxKind::LOCAL_LABEL)
                                                    .collect(),
                                            ),
                                        )
                                    }),
                                _ => None,
                            })
                        })
                        .collect()
                })
                .ok_or_else(|| lsp_error_map(ErrorCode::NoRoot))?,
        ))
    }
}

fn get_numeric_hover(value: &NumericToken) -> Option<Vec<String>> {
    let value = value.value();
    Some(vec![
        "# Number".to_string(),
        format!("Decimal: {}", value),
        format!("Hex: {:#X}", value),
    ])
}

fn get_label_hover(label: &LabelToken) -> Option<Vec<String>> {
    let mut symbols = Vec::new();

    if let Some((sym, lang)) = label.demangle() {
        symbols.push(String::from("# Demangled Symbol\n"));
        symbols.push(format!("**{}**: `{}`", lang, sym));
    }

    Some(symbols)
}

fn find_references<'a>(
    parser: &'a Parser,
    token: &'a SyntaxToken,
) -> impl Iterator<Item = SyntaxToken> + 'a {
    parser
        .tree()
        .descendants_with_tokens()
        .filter_map(|d| d.into_token())
        .filter(move |t| parser.token_text_equal(token, t))
}

fn node_to_document_symbol(
    position: &PositionInfo,
    node: &SyntaxNode,
    token: &SyntaxToken,
    child: Option<Vec<SyntaxNode>>,
) -> DocumentSymbol {
    DocumentSymbol {
        name: token.text().to_string(),
        detail: None,
        kind: SymbolKind::Function,
        tags: None,
        deprecated: None,
        range: position.range_for_node(node).unwrap().into(),
        selection_range: position.range_for_node(node).unwrap().into(),
        children: child.map(|c| {
            c.iter()
                .map(|node| {
                    node_to_document_symbol(
                        position,
                        node,
                        &find_token(node, SyntaxKind::LABEL).unwrap(),
                        None,
                    )
                })
                .collect()
        }),
    }
}

fn find_token(node: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
    node.children_with_tokens()
        .filter_map(|c| c.into_token())
        .find(|c| c.kind() == kind)
}

fn find_nodes(node: &SyntaxNode, kind: SyntaxKind) -> impl std::iter::Iterator<Item = SyntaxNode> {
    node.children().filter(move |node| node.kind() == kind)
}

impl AssemblyLanguageServerProtocol {
    pub fn new(data: &str, uri: Url, config: LSPConfig) -> Self {
        let parser = Parser::from(data, &config);
        Self {
            parser,
            uri,
            config,
        }
    }
}

#[cfg(test)]
mod tests {
    use lsp_types::{
        DocumentSymbol, DocumentSymbolResponse, GotoDefinitionResponse, Hover, MarkedString,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_updates() {
        let mut actor = AssemblyLanguageServerProtocol::new(
            r#"stp	x29, x30, [sp, -32]!
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            Default::default(),
        );

        let orig_data = actor.parser.clone();
        actor.update("stp x20, x21, [sp, -32]!");
        assert_ne!(orig_data, actor.parser);
    }

    #[test]
    fn test_goto_definition_with_label() {
        let actor = AssemblyLanguageServerProtocol::new(
            r#"entry:
    b entry
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            Default::default(),
        );

        let actual = GotoDefinitionResponse::Array(vec![Location {
            uri: Url::parse("file://temp").unwrap(),
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 6),
            },
        }]);

        let response = actor
            .goto_definition(DocumentPosition { line: 1, column: 8 })
            .unwrap();

        assert_eq!(actual, response);
    }

    #[test]
    fn test_goto_definition_with_label_not_first() {
        let actor = AssemblyLanguageServerProtocol::new(
            r#"b entry
entry:
    b entry
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            Default::default(),
        );

        let actual = GotoDefinitionResponse::Array(vec![Location {
            uri: Url::parse("file://temp").unwrap(),
            range: Range {
                start: Position::new(1, 0),
                end: Position::new(1, 6),
            },
        }]);

        let response = actor
            .goto_definition(DocumentPosition { line: 2, column: 8 })
            .unwrap();

        assert_eq!(actual, response);
    }

    #[test]
    fn test_goto_definition_with_label_not_defined() {
        let actor = AssemblyLanguageServerProtocol::new(
            r#"entry:
    b somewhere
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            Default::default(),
        );

        let actual = GotoDefinitionResponse::Array(vec![]);

        let response = actor
            .goto_definition(DocumentPosition { line: 1, column: 8 })
            .unwrap();

        assert_eq!(actual, response);
    }

    #[test]
    fn test_goto_definition_on_opcode() {
        let actor = AssemblyLanguageServerProtocol::new(
            r#"entry:
    stp x20, x21, [sp, -32]!
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            Default::default(),
        );

        let actual = GotoDefinitionResponse::Array(vec![]);

        let response = actor
            .goto_definition(DocumentPosition { line: 1, column: 6 })
            .unwrap();

        assert_eq!(actual, response);
    }

    #[test]
    fn test_goto_definition_with_not_on_token() {
        let actor = AssemblyLanguageServerProtocol::new(
            r#"entry:
    stp x20, x21, [sp, -32]!
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            Default::default(),
        );

        let actual = GotoDefinitionResponse::Array(vec![]);

        let response = actor
            .goto_definition(DocumentPosition { line: 1, column: 7 })
            .unwrap();

        assert_eq!(actual, response);
    }

    #[test]
    fn test_find_references() {
        let actor = AssemblyLanguageServerProtocol::new(
            r#"entry:
    b entry
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            Default::default(),
        );

        let actual = vec![Location {
            uri: Url::parse("file://temp").unwrap(),
            range: Range {
                start: Position::new(1, 6),
                end: Position::new(1, 11),
            },
        }];

        let response = actor
            .find_references(DocumentPosition { line: 1, column: 8 }, false)
            .unwrap();

        assert_eq!(actual, response);
    }

    #[test]
    fn test_find_references_numeric() {
        let actor = AssemblyLanguageServerProtocol::new(
            r#"entry:
.cfi_startproc
    stp x20, x21, [sp, -32]!
.L2:
    b .L2
end:
.cfi_endproc

// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            Default::default(),
        );

        let actual = actor
            .find_references(
                DocumentPosition {
                    line: 2,
                    column: 25,
                },
                false,
            )
            .unwrap();
        let response: Vec<Location> = vec![];

        assert_eq!(response, actual);
    }

    #[test]
    fn test_find_references_instruction() {
        let actor = AssemblyLanguageServerProtocol::new(
            r#"entry:
.cfi_startproc
    stp x20, x21, [sp, -32]!
.L2:
    b .L2
end:
.cfi_endproc

// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            Default::default(),
        );

        let actual = actor
            .find_references(DocumentPosition { line: 3, column: 5 }, false)
            .unwrap();
        let response: Vec<Location> = vec![];

        assert_eq!(response, actual);
    }

    #[test]
    fn test_document_hover_numeric() {
        let actor = AssemblyLanguageServerProtocol::new(
            r#"entry:
.cfi_startproc
    stp x20, x21, [sp, -32]!
.L2:
    b .L2
end:
.cfi_endproc

// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            Default::default(),
        );

        let actual = actor
            .hover(DocumentPosition {
                line: 2,
                column: 25,
            })
            .unwrap();
        let response = Some(Hover {
            contents: lsp_types::HoverContents::Markup(MarkupContent {
                value: r#"# Number
Decimal: -32
Hex: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE0"#
                    .to_string(),
                kind: lsp_types::MarkupKind::Markdown,
            }),
            range: None,
        });

        assert_eq!(response, actual);
    }

    #[test]
    fn test_document_symbols() {
        let actor = AssemblyLanguageServerProtocol::new(
            r#"entry:
.cfi_startproc
    stp x20, x21, [sp, -32]!
.L2:
    b .L2
end:
.cfi_endproc

// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            Default::default(),
        );

        let actual = DocumentSymbolResponse::Nested(
            [
                DocumentSymbol {
                    name: "entry:".to_string(),
                    detail: None,
                    kind: SymbolKind::Function,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 5,
                            character: 0,
                        },
                    },
                    selection_range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 5,
                            character: 0,
                        },
                    },
                    children: Some(vec![DocumentSymbol {
                        name: ".L2:".to_string(),
                        detail: None,
                        kind: SymbolKind::Function,
                        tags: None,
                        deprecated: None,
                        range: Range {
                            start: Position {
                                line: 3,
                                character: 0,
                            },
                            end: Position {
                                line: 5,
                                character: 0,
                            },
                        },
                        selection_range: Range {
                            start: Position {
                                line: 3,
                                character: 0,
                            },
                            end: Position {
                                line: 5,
                                character: 0,
                            },
                        },
                        children: None,
                    }]),
                },
                DocumentSymbol {
                    name: "end:".to_string(),
                    detail: None,
                    kind: SymbolKind::Function,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position {
                            line: 5,
                            character: 0,
                        },
                        end: Position {
                            line: 8,
                            character: 32,
                        },
                    },
                    selection_range: Range {
                        start: Position {
                            line: 5,
                            character: 0,
                        },
                        end: Position {
                            line: 8,
                            character: 32,
                        },
                    },
                    children: Some(vec![]),
                },
            ]
            .to_vec(),
        );

        let response = actor.document_symbols().unwrap();

        assert_eq!(actual, response);
    }

    #[test]
    fn test_document_highlight_label() {
        let actor = AssemblyLanguageServerProtocol::new(
            r#"entry:
    b entry
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            Default::default(),
        );

        let actual = vec![
            DocumentHighlight {
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 6),
                },
                kind: Some(DocumentHighlightKind::Text),
            },
            DocumentHighlight {
                range: Range {
                    start: Position::new(1, 6),
                    end: Position::new(1, 11),
                },
                kind: Some(DocumentHighlightKind::Text),
            },
        ];

        let response = actor
            .document_highlight(DocumentPosition { line: 1, column: 8 })
            .unwrap();

        assert_eq!(actual, response);
    }

    #[test]
    fn test_document_semantic() {
        {
            let actor = AssemblyLanguageServerProtocol::new(
                r#"entry:
    stp	x29, x30, [sp, -32]!
    b entry
// lsp-asm-architecture: AArch64"#,
                Url::parse("file://temp").unwrap(),
                Default::default(),
            );

            let actual = SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: vec![
                    SemanticToken {
                        delta_line: 0,
                        delta_start: 0,
                        length: 6,
                        token_type: 6,
                        token_modifiers_bitset: 0,
                    },
                    SemanticToken {
                        delta_line: 1,
                        delta_start: 4,
                        length: 3,
                        token_type: 0,
                        token_modifiers_bitset: 0,
                    },
                    SemanticToken {
                        delta_line: 0,
                        delta_start: 4,
                        length: 3,
                        token_type: 8,
                        token_modifiers_bitset: 0,
                    },
                    SemanticToken {
                        delta_line: 0,
                        delta_start: 5,
                        length: 3,
                        token_type: 8,
                        token_modifiers_bitset: 0,
                    },
                    SemanticToken {
                        delta_line: 0,
                        delta_start: 6,
                        length: 2,
                        token_type: 5,
                        token_modifiers_bitset: 0,
                    },
                    SemanticToken {
                        delta_line: 0,
                        delta_start: 4,
                        length: 3,
                        token_type: 2,
                        token_modifiers_bitset: 0,
                    },
                    SemanticToken {
                        delta_line: 1,
                        delta_start: 4,
                        length: 1,
                        token_type: 0,
                        token_modifiers_bitset: 0,
                    },
                    SemanticToken {
                        delta_line: 1,
                        delta_start: 0,
                        length: 32,
                        token_type: 4,
                        token_modifiers_bitset: 0,
                    },
                ],
            });

            let response = actor.get_semantic_tokens(None).unwrap();

            assert_eq!(actual, response);
        }
    }
}
