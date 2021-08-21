#![allow(deprecated)]
use itertools::*;
use std::iter;

use lsp_server::ResponseError;
use lsp_types::{
    CodeLens, Command, DocumentHighlightKind, DocumentSymbol, DocumentSymbolResponse,
    HoverContents, Location, MarkupContent, Range, SemanticToken, SemanticTokens,
    SemanticTokensResult, SymbolKind, Url,
};
use rowan::TextRange;

use crate::config::LSPConfig;
use crate::documentation::{self, load_documentation};
use crate::handler::error::{lsp_error_map, ErrorCode};
use crate::handler::semantic::semantic_delta_transform;
use crate::handler::types::DocumentChange;
use crate::handler::LanguageServerProtocol;
use crate::types::{Architecture, DocumentPosition, DocumentRange};

use super::ast::{
    self, find_kind_index, AstNode, LabelNode, LabelToken, LocalLabelNode, NumericToken,
    RegisterToken, SyntaxKind, SyntaxToken,
};
use super::llvm_mca::run_mca;
use super::parser::{Parser, PositionInfo};
use super::registers::{registers_for_architecture, RegisterKind};

pub struct AssemblyLanguageServerProtocol {
    parser: Parser,
    uri: Url,
    config: LSPConfig,
    version: u32,
}

impl LanguageServerProtocol for AssemblyLanguageServerProtocol {
    fn update(&mut self, version: u32, changes: Vec<DocumentChange>) -> bool {
        if self.version >= version {
            error!(
                "Invalid update requested version {} to {}",
                self.version, version
            );
            return false;
        }

        let mut contents = self.parser.reconstruct_file();
        for change in changes {
            self.apply_change(&mut contents, change);
        }
        self.version = version;
        self.parser = Parser::from(contents.as_str(), &self.config);
        true
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
        let range = self.parser.tree().text_range();
        let references = find_references(&self.parser, &token, &range);
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
            SyntaxKind::MNEMONIC => get_hover_mnemonic(&token, self.parser.architecture()),
            SyntaxKind::L_PAREN
            | SyntaxKind::R_PAREN
            | SyntaxKind::L_SQ
            | SyntaxKind::R_SQ
            | SyntaxKind::L_CURLY
            | SyntaxKind::R_CURLY
            | SyntaxKind::L_ANGLE
            | SyntaxKind::R_ANGLE
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

        Ok(hover.map(|hover| lsp_types::Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: lsp_types::MarkupKind::Markdown,
                value: hover.join("  \n"),
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

        let references = find_references(&self.parser, &token, &range);
        let locations = references
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
                        .token::<RegisterToken>(&token)
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
                    let pos = position.get_position(&token)?;
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
        let position = self.parser.position();
        Ok(DocumentSymbolResponse::Nested(
            self.parser
                .tree()
                .descendants()
                .filter_map(|n| {
                    LabelNode::cast(&n)
                        .map(|label| label.to_document_symbol(position))
                        .flatten()
                })
                .collect::<Vec<_>>(),
        ))
    }

    fn code_lens(&self) -> Result<Option<Vec<lsp_types::CodeLens>>, ResponseError> {
        if self.parser.filesize() > self.config.codelens.enabled_filesize {
            info!(
                "Skipping codelens due to filesize threshold see codelens::enabled_filesize ({}) config", self.config.codelens.enabled_filesize
            );
            return Ok(None);
        }

        let map = self.parser.debug_map();
        let lens = (self.config.codelens.loc_enabled && map.has_debug_map()).then(|| {
            self.parser
                .tree()
                .descendants()
                .filter(|d| matches!(d.kind(), SyntaxKind::DIRECTIVE))
                .filter(|d| {
                    ast::find_kind_index(d, 0, SyntaxKind::MNEMONIC)
                        .map(|t| t.as_token().map(|t| t.text() == ".loc"))
                        .flatten()
                        .unwrap_or(false)
                })
                .filter_map(|n| {
                    let location = map.get_location(&n)?;
                    let title = map.get_contents(location)?.clone();
                    let range = self.parser.position().range_for_node(&n)?.into();
                    let location: Location = map.get_file_location(&n).map(|l| l.into())?;

                    Some(CodeLens {
                        range,
                        command: Some(Command {
                            title,
                            command: String::from("lsp-asm.loc"),
                            arguments: Some(vec![serde_json::to_value(location).unwrap()]),
                        }),
                        data: None,
                    })
                })
                .collect()
        });
        Ok(lens)
    }

    fn syntax_tree(&self) -> Result<String, ResponseError> {
        Ok(format!("{:#?}", self.parser.tree()))
    }

    fn analysis(&self, range: Option<DocumentRange>) -> Result<String, ResponseError> {
        let range = range
            .map(|r| self.parser.position().range_to_text_range(&r))
            .flatten()
            .unwrap_or_else(|| self.parser.tree().text_range());

        let tokens = self.parser.tokens_in_range(&range).filter(|t| {
            !(matches!(t.kind(), SyntaxKind::METADATA)
                || ast::find_parent(t, SyntaxKind::METADATA).is_some())
        });
        let asm = self.parser.reconstruct_from_tokens(tokens, &range);
        run_mca(
            asm.as_str(),
            self.parser.architecture(),
            &self.config.analysis,
        )
        .map_err(|e| lsp_error_map(ErrorCode::MCAFailed(e.to_string())))
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

fn get_hover_mnemonic(token: &SyntaxToken, arch: &Architecture) -> Option<Vec<String>> {
    let docs = load_documentation(arch).ok()?;
    let instructions = docs.get(&token.text().to_lowercase())?;

    let template = documentation::find_correct_instruction_template(
        &token.parent(),
        instructions,
        &registers_for_architecture(arch),
    );

    if let Some(template) = template {
        let instruction = documentation::instruction_from_template(instructions, template)?;

        Some(vec![format!("{}", instruction)])
    } else {
        // Couldn't resolve which instruction we are on so print them all.
        Some(
            instructions
                .iter()
                .map(|i| format!("{}", i))
                .interleave_shortest(iter::repeat(String::from("---")))
                .collect(),
        )
    }
}

fn find_references<'a>(
    parser: &'a Parser,
    token: &'a SyntaxToken,
    range: &'a TextRange,
) -> impl Iterator<Item = SyntaxToken> + 'a {
    parser
        .tokens_in_range(range)
        .filter(move |t| parser.token_text_equal(token, t))
}

impl<'s> LabelNode<'s> {
    fn to_document_symbol(&self, position: &PositionInfo) -> Option<DocumentSymbol> {
        let token = find_kind_index(self.syntax(), 1, SyntaxKind::LABEL)?.into_token()?;
        let node = self.syntax();

        Some(DocumentSymbol {
            name: token.text().to_string(),
            detail: None,
            kind: SymbolKind::Function,
            tags: None,
            deprecated: None,
            range: position.range_for_node(node).unwrap().into(),
            selection_range: position.range_for_node(node).unwrap().into(),
            children: self
                .sub_labels()
                .map(|s| {
                    LocalLabelNode::cast(&s)
                        .map(|s| s.to_document_symbol(position))
                        .flatten()
                })
                .collect(),
        })
    }
}

impl<'s> LocalLabelNode<'s> {
    fn to_document_symbol(&self, position: &PositionInfo) -> Option<DocumentSymbol> {
        let token = find_kind_index(self.syntax(), 0, SyntaxKind::LABEL)?.into_token()?;
        let node = self.syntax();

        Some(DocumentSymbol {
            name: token.text().to_string(),
            detail: None,
            kind: SymbolKind::Function,
            tags: None,
            deprecated: None,
            range: position.range_for_node(node).unwrap().into(),
            selection_range: position.range_for_node(node).unwrap().into(),
            children: None,
        })
    }
}

impl AssemblyLanguageServerProtocol {
    pub fn new(data: &str, uri: Url, version: u32, config: LSPConfig) -> Self {
        let parser = Parser::from(data, &config);
        Self {
            parser,
            uri,
            config,
            version,
        }
    }

    fn apply_change(&self, contents: &mut String, change: DocumentChange) {
        if let Some(range) = &change.range {
            let range = {
                let start_position = self
                    .parser
                    .position()
                    .offset_for_line(range.start.line)
                    .unwrap();
                let start_offset = (start_position + range.start.column) as usize;

                let end_position = self
                    .parser
                    .position()
                    .offset_for_line(range.end.line)
                    .unwrap();
                let end_offset = (end_position + range.end.column) as usize;

                start_offset..end_offset
            };

            contents.replace_range(range, &change.text);
        } else {
            *contents = change.text;
        }
    }
}

#[cfg(test)]
mod tests {
    use lsp_types::{
        DocumentHighlight, DocumentSymbol, DocumentSymbolResponse, GotoDefinitionResponse, Position,
    };
    use pretty_assertions::assert_eq;

    use crate::types::DocumentRange;

    use super::*;

    #[test]
    fn test_goto_definition_with_label() {
        let actor = AssemblyLanguageServerProtocol::new(
            r#"entry:
    b entry
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
            Default::default(),
        );

        let actual = actor
            .find_references(DocumentPosition { line: 3, column: 5 }, false)
            .unwrap();
        let response: Vec<Location> = vec![];

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
            0,
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
            0,
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
                0,
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

    #[test]
    fn test_invalid_versions() {
        let mut lsp = AssemblyLanguageServerProtocol::new(
            "str x1, [sp, #80]",
            Url::parse("file://test").unwrap(),
            0,
            LSPConfig {
                architecture: crate::types::Architecture::AArch64,
                ..Default::default()
            },
        );

        assert_eq!(
            true,
            lsp.update(
                5,
                vec![DocumentChange {
                    text: String::from("// test"),
                    range: Some(DocumentRange {
                        start: DocumentPosition { line: 0, column: 0 },
                        end: DocumentPosition { line: 0, column: 0 },
                    }),
                }],
            )
        );
        assert_eq!(
            false,
            lsp.update(
                5,
                vec![DocumentChange {
                    text: String::from("// te"),
                    range: Some(DocumentRange {
                        start: DocumentPosition { line: 0, column: 0 },
                        end: DocumentPosition { line: 0, column: 0 },
                    }),
                }],
            )
        );
        assert_eq!(
            false,
            lsp.update(
                3,
                vec![DocumentChange {
                    text: String::from("// te"),
                    range: Some(DocumentRange {
                        start: DocumentPosition { line: 0, column: 0 },
                        end: DocumentPosition { line: 0, column: 0 },
                    }),
                }],
            )
        );

        assert_eq!(
            true,
            lsp.update(
                6,
                vec![DocumentChange {
                    text: String::from("// test more"),
                    range: Some(DocumentRange {
                        start: DocumentPosition { line: 0, column: 0 },
                        end: DocumentPosition { line: 0, column: 0 },
                    }),
                }],
            )
        );
    }
}
