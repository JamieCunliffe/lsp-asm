#![allow(deprecated)]
use std::path::PathBuf;
use std::sync::Arc;

use super::ast::{AstNode, LabelNode, LocalLabelNode, RegisterToken};
use super::llvm_mca::run_mca;
use super::parser::{split_parsed_include, Parser, PositionInfo};
use super::{definition, references};
use crate::asm::{hovers, inlay_hints, signature};
use crate::completion;
use crate::handler::context::Context;
use crate::handler::error::{lsp_error_map, ErrorCode};
use crate::handler::semantic::semantic_delta_transform;
use crate::handler::types::DocumentChange;
use crate::types::{DocumentPosition, DocumentRange};

use arch::registers::registers_for_architecture;
use base::register::RegisterKind;
use base::rwlock::RwLock;
use documentation::access::access_type;
use documentation::OperandAccessType;
use fmt::FormatOptions;
use itertools::*;
use lsp_server::ResponseError;
use lsp_types::{
    AnnotatedTextEdit, CodeActionOrCommand, CodeLens, Command, CompletionList,
    DocumentHighlightKind, DocumentSymbol, DocumentSymbolResponse, HoverContents, InlayHint,
    Location, MarkupContent, OneOf, OptionalVersionedTextDocumentIdentifier, Range, SemanticToken,
    SemanticTokens, SemanticTokensResult, SignatureHelp, SymbolKind, TextDocumentEdit, TextEdit,
    Url, WorkspaceEdit,
};
use parser::ParsedInclude;
use rowan::TextRange;
use syntax::ast::{self, find_kind_index, find_parent, SyntaxKind, SyntaxToken};
use syntax::utils::token_is_local_label;

pub struct AssemblyLanguageServerProtocol {
    parser: Parser,
    uri: Url,
    version: u32,
}

impl AssemblyLanguageServerProtocol {
    pub fn new(context: Arc<Context>, data: &str, uri: Url, version: u32) -> Self {
        let (parser, includes) = Parser::from(uri.clone(), data, context.config());

        context.add_actors(handle_includes(includes, context.clone(), &uri));

        Self {
            parser,
            uri,
            version,
        }
    }

    pub fn update(
        &mut self,
        context: Arc<Context>,
        version: u32,
        changes: Vec<DocumentChange>,
    ) -> Result<Vec<(Url, RwLock<Self>)>, ResponseError> {
        if self.version >= version {
            error!(
                "Invalid update requested version {} to {}",
                self.version, version
            );
            return Err(lsp_error_map(ErrorCode::InvalidVersion(self.uri.clone())));
        }

        let mut contents = self.parser.reconstruct_file();
        for change in changes {
            self.apply_change(&mut contents, change);
        }
        self.version = version;
        let (parser, includes) =
            Parser::from(self.uri.clone(), contents.as_str(), context.config());
        self.parser = parser;

        Ok(handle_includes(includes, context, &self.uri))
    }

    pub fn parser(&self) -> &Parser {
        &self.parser
    }

    pub(crate) fn version(&self) -> u32 {
        self.version
    }

    pub fn goto_definition(
        &self,
        context: Arc<Context>,
        position: DocumentPosition,
    ) -> Result<lsp_types::GotoDefinitionResponse, lsp_server::ResponseError> {
        let token = self
            .parser
            .token_at_point(&position)
            .ok_or_else(|| lsp_error_map(ErrorCode::TokenNotFound))?;

        let get_mnemonic = || {
            find_parent(&token, SyntaxKind::DIRECTIVE)
                .or_else(|| find_parent(&token, SyntaxKind::INSTRUCTION))
                .and_then(|n| find_kind_index(&n, 0, SyntaxKind::MNEMONIC))
                .and_then(|token| token.into_token())
        };

        let res = match token.kind() {
            SyntaxKind::TOKEN => definition::goto_definition_label(context, &self.parser, &token)?,
            SyntaxKind::MNEMONIC if token.text() == ".loc" => {
                definition::goto_definition_loc(&self.parser, &token)?
            }
            SyntaxKind::MNEMONIC if syntax::utils::is_token_include(token.text()) => {
                definition::goto_definition_label_include(&token, &self.uri)?
            }
            SyntaxKind::CONSTANT => {
                definition::goto_definition_const(context, &token, &self.parser)?
            }
            SyntaxKind::REGISTER_ALIAS => {
                definition::goto_definition_reg_alias(context, &token, &self.parser)?
            }
            _ if get_mnemonic()
                .map(|token| syntax::utils::is_token_include(token.text()))
                .unwrap_or(false) =>
            {
                definition::goto_definition_label_include(&token, &self.uri)?
            }
            _ => Vec::new(),
        };

        Ok(lsp_types::GotoDefinitionResponse::Array(res))
    }

    pub fn find_references(
        &self,
        context: Arc<Context>,
        position: DocumentPosition,
        include_decl: bool,
    ) -> Result<Vec<Location>, lsp_server::ResponseError> {
        let token = self
            .parser
            .token_at_point(&position)
            .ok_or_else(|| lsp_error_map(ErrorCode::TokenNotFound))?;
        let range = references::get_search_range(&self.parser, &token, None);
        let position = self.parser.position();
        let references = references::find_references(&self.parser, &token, range, include_decl);
        let included_files = if token_is_local_label(&token) {
            None
        } else {
            let handle_related = |parser: &Parser| {
                let id = parser.uri();
                let range = parser.text_range();
                let position = parser.position();

                references::find_references(parser, &token, range, include_decl)
                    .filter_map(move |token| {
                        Some(lsp_types::Location::new(
                            id.clone(),
                            position.range_for_token(&token)?.into(),
                        ))
                    })
                    .collect_vec()
            };

            let related = context.related_parsers(false, self.uri.clone(), |parser| {
                handle_related(parser).into_iter()
            });

            Some(related)
        };

        Ok(references
            .filter_map(move |token| {
                Some(lsp_types::Location::new(
                    self.uri.clone(),
                    position.range_for_token(&token)?.into(),
                ))
            })
            .chain(included_files.into_iter().flatten())
            .collect())
    }

    pub fn hover(
        &self,
        context: Arc<Context>,
        position: DocumentPosition,
    ) -> Result<Option<lsp_types::Hover>, lsp_server::ResponseError> {
        let token = self
            .parser
            .token_at_point(&position)
            .ok_or_else(|| lsp_error_map(ErrorCode::TokenNotFound))?;

        let hover = match token.kind() {
            SyntaxKind::TOKEN => hovers::get_token_hover(context, &self.parser, token),
            SyntaxKind::NUMBER => hovers::get_numeric_hover(
                &self
                    .parser
                    .token(&token)
                    .ok_or_else(|| lsp_error_map(ErrorCode::CastFailed))?,
            ),
            SyntaxKind::LABEL => hovers::get_label_hover(
                &self
                    .parser
                    .token(&token)
                    .ok_or_else(|| lsp_error_map(ErrorCode::CastFailed))?,
            ),
            SyntaxKind::MNEMONIC => {
                hovers::get_hover_mnemonic(&token, self.parser.architecture(), self.parser.alias())
            }
            SyntaxKind::REGISTER_ALIAS => hovers::get_alias_hover(&token, self.parser.alias()),
            SyntaxKind::CONSTANT => hovers::get_constant_hover(&token, self.parser.alias()),
            SyntaxKind::L_PAREN
            | SyntaxKind::R_PAREN
            | SyntaxKind::L_SQ
            | SyntaxKind::R_SQ
            | SyntaxKind::L_CURLY
            | SyntaxKind::R_CURLY
            | SyntaxKind::L_ANGLE
            | SyntaxKind::R_ANGLE
            | SyntaxKind::REGISTER
            | SyntaxKind::WHITESPACE
            | SyntaxKind::COMMA
            | SyntaxKind::OPERATOR
            | SyntaxKind::STRING
            | SyntaxKind::LOCAL_LABEL
            | SyntaxKind::COMMENT
            | SyntaxKind::IMMEDIATE
            | SyntaxKind::FLOAT
            | SyntaxKind::ALIAS
            | SyntaxKind::RELOCATION
            | SyntaxKind::INSTRUCTION
            | SyntaxKind::DIRECTIVE
            | SyntaxKind::BRACKETS
            | SyntaxKind::METADATA
            | SyntaxKind::OBJDUMP_OFFSET
            | SyntaxKind::CONST_DEF
            | SyntaxKind::EXPR
            | SyntaxKind::NAME
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

    pub fn code_actions(
        &self,
        _context: Arc<Context>,
        range: DocumentRange,
    ) -> Result<Vec<lsp_types::CodeActionOrCommand>, lsp_server::ResponseError> {
        let pos = self.parser.position();
        let start = pos
            .point_for_position(&range.start)
            .ok_or_else(|| lsp_error_map(ErrorCode::InvalidPosition))?;
        let end = pos
            .point_for_position(&range.end)
            .ok_or_else(|| lsp_error_map(ErrorCode::InvalidPosition))?;

        let actions =
            codeactions::code_actions(*self.parser.architecture(), self.parser.tree(), start, end);

        Ok(actions
            .into_iter()
            .filter_map(|action| {
                Some(CodeActionOrCommand::CodeAction(lsp_types::CodeAction {
                    title: action.name,
                    kind: None,
                    diagnostics: None,
                    edit: Some(WorkspaceEdit {
                        changes: None,
                        document_changes: Some(lsp_types::DocumentChanges::Edits(vec![
                            TextDocumentEdit {
                                text_document: OptionalVersionedTextDocumentIdentifier {
                                    uri: self.uri.clone(),
                                    version: None,
                                },
                                edits: action
                                    .edit
                                    .into_iter()
                                    .map(
                                        |edit| -> Result<
                                            OneOf<TextEdit, AnnotatedTextEdit>,
                                            lsp_server::ResponseError,
                                        > {
                                            Ok(OneOf::Left(lsp_types::TextEdit {
                                                range: Range::new(
                                                    pos.get_position_for_size(&edit.start)
                                                        .ok_or_else(|| {
                                                            lsp_error_map(
                                                                ErrorCode::InvalidPosition,
                                                            )
                                                        })?
                                                        .into(),
                                                    pos.get_position_for_size(&edit.end)
                                                        .ok_or_else(|| {
                                                            lsp_error_map(
                                                                ErrorCode::InvalidPosition,
                                                            )
                                                        })?
                                                        .into(),
                                                ),
                                                new_text: edit.text,
                                            }))
                                        },
                                    )
                                    .try_collect()
                                    .ok()?,
                            },
                        ])),
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: None,
                    disabled: None,
                    data: None,
                }))
            })
            .collect())
    }

    pub fn document_highlight(
        &self,
        _context: Arc<Context>,
        position: DocumentPosition,
    ) -> Result<Vec<lsp_types::DocumentHighlight>, lsp_server::ResponseError> {
        let token = self
            .parser
            .token_at_point(&position)
            .ok_or_else(|| lsp_error_map(ErrorCode::TokenNotFound))?;

        let position_cache = self.parser.position();

        let range = references::get_search_range(&self.parser, &token, Some(200));

        let docs = documentation::load_documentation(self.parser.architecture()).ok();
        let registers = registers_for_architecture(self.parser.architecture());

        let references = references::find_references(&self.parser, &token, range, true);

        let to_proto_kind = |k: OperandAccessType| match k {
            OperandAccessType::Unknown | OperandAccessType::Text => DocumentHighlightKind::TEXT,
            OperandAccessType::Read => DocumentHighlightKind::READ,
            OperandAccessType::Write => DocumentHighlightKind::WRITE,
        };

        let locations = if let Some(docs) = docs {
            references
                .filter_map(|token| {
                    let kind = access_type(
                        &token,
                        &docs,
                        registers,
                        self.parser.alias(),
                        *self.parser.architecture(),
                    )
                    .map(to_proto_kind)
                    .or(Some(DocumentHighlightKind::TEXT));

                    Some(lsp_types::DocumentHighlight {
                        range: position_cache.range_for_token(&token)?.into(),
                        kind,
                    })
                })
                .collect()
        } else {
            references
                .filter_map(|token| {
                    Some(lsp_types::DocumentHighlight {
                        range: position_cache.range_for_token(&token)?.into(),
                        kind: Some(DocumentHighlightKind::TEXT),
                    })
                })
                .collect()
        };

        Ok(locations)
    }

    pub fn get_semantic_tokens(
        &self,
        _context: Arc<Context>,
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
            self.parser.text_range()
        };

        let position = self.parser.position();
        let tokens = self.parser.tokens_in_range(range);

        let tokens = tokens
            .filter_map(|token| {
                if let Some(index) = match token.kind() {
                    _ if syntax::ast::find_parent(&token, SyntaxKind::METADATA).is_some() => {
                        Some(crate::handler::semantic::METADATA_INDEX)
                    }
                    SyntaxKind::METADATA | SyntaxKind::OBJDUMP_OFFSET => {
                        Some(crate::handler::semantic::METADATA_INDEX)
                    }
                    SyntaxKind::MNEMONIC => match token.parent()?.kind() {
                        SyntaxKind::INSTRUCTION => Some(crate::handler::semantic::OPCODE_INDEX),
                        SyntaxKind::DIRECTIVE | SyntaxKind::ALIAS | SyntaxKind::CONST_DEF => {
                            Some(crate::handler::semantic::DIRECTIVE_INDEX)
                        }
                        _ => unreachable!("Invalid parent kind"),
                    },
                    SyntaxKind::COMMENT => Some(crate::handler::semantic::COMMENT_INDEX),
                    SyntaxKind::NUMBER | SyntaxKind::FLOAT => {
                        Some(crate::handler::semantic::NUMERIC_INDEX)
                    }
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
                    SyntaxKind::REGISTER_ALIAS => {
                        let register = self.parser.alias().get_register_for_alias(token.text())?;
                        let registers = registers_for_architecture(self.parser.architecture());

                        let kind = registers.get_kind(register);
                        if kind.contains(RegisterKind::GENERAL_PURPOSE) {
                            Some(crate::handler::semantic::GP_REGISTER_INDEX)
                        } else if kind.contains(RegisterKind::FLOATING_POINT) {
                            Some(crate::handler::semantic::FP_REGISTER_INDEX)
                        } else {
                            Some(crate::handler::semantic::REGISTER_INDEX)
                        }
                    }
                    SyntaxKind::LABEL | SyntaxKind::LOCAL_LABEL => {
                        Some(crate::handler::semantic::LABEL_INDEX)
                    }
                    SyntaxKind::RELOCATION => Some(crate::handler::semantic::RELOCATION_INDEX),
                    SyntaxKind::CONSTANT => Some(crate::handler::semantic::CONSTANT_INDEX),
                    SyntaxKind::NAME
                        if syntax::ast::find_parent(&token, SyntaxKind::CONST_DEF).is_some() =>
                    {
                        Some(crate::handler::semantic::CONSTANT_INDEX)
                    }
                    SyntaxKind::NAME | SyntaxKind::CONST_DEF | SyntaxKind::EXPR => None,
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
                    | SyntaxKind::ALIAS
                    | SyntaxKind::IMMEDIATE
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

    pub fn document_symbols(
        &self,
        _context: Arc<Context>,
    ) -> Result<lsp_types::DocumentSymbolResponse, lsp_server::ResponseError> {
        let position = self.parser.position();
        Ok(DocumentSymbolResponse::Nested(
            self.parser
                .tree()
                .descendants()
                .filter_map(|n| {
                    LabelNode::cast(&n).and_then(|label| label.to_document_symbol(position))
                })
                .collect::<Vec<_>>(),
        ))
    }

    pub fn code_lens(
        &self,
        context: Arc<Context>,
    ) -> Result<Option<Vec<lsp_types::CodeLens>>, ResponseError> {
        if self.parser.filesize() > context.config().codelens.enabled_filesize {
            info!(
                "Skipping codelens due to filesize threshold see codelens::enabled_filesize ({}) config", context.config().codelens.enabled_filesize
            );
            return Ok(None);
        }

        let map = self.parser.debug_map();
        let lens = (context.config().codelens.loc_enabled && map.has_debug_map()).then(|| {
            self.parser
                .tree()
                .descendants()
                .filter(|d| matches!(d.kind(), SyntaxKind::DIRECTIVE))
                .filter(|d| {
                    ast::find_kind_index(d, 0, SyntaxKind::MNEMONIC)
                        .and_then(|t| t.as_token().map(|t| t.text() == ".loc"))
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

    pub fn inlay_hint(
        &self,
        _context: Arc<Context>,
        location: Option<DocumentRange>,
    ) -> Result<Option<Vec<InlayHint>>, ResponseError> {
        let parser = self.parser();
        let position = parser.position();
        let range =
            location.map(|range| position.make_range_for_lines(range.start.line, range.end.line));

        if matches!(self.parser().file_type(), base::FileType::ObjDump(opts) if opts.show_leading_addr)
        {
            return Ok(Some(inlay_hints::objdump_inlay_hints(
                parser,
                range.unwrap_or_else(|| parser.text_range()),
            )));
        }

        Ok(None)
    }

    pub fn completion(
        &self,
        _context: Arc<Context>,
        location: DocumentPosition,
    ) -> Result<CompletionList, ResponseError> {
        let docs = documentation::load_documentation(self.parser.architecture());

        let items = completion::handle_completion(&self.parser, &location, docs.ok())
            .unwrap_or_default()
            .into_iter()
            .map(|i| i.into())
            .collect_vec();

        Ok(CompletionList {
            is_incomplete: true,
            items,
        })
    }

    pub fn signature_help(
        &self,
        _context: Arc<Context>,
        position: &DocumentPosition,
    ) -> Result<Option<SignatureHelp>, ResponseError> {
        let location = self
            .parser
            .position()
            .point_for_position(position)
            .ok_or_else(|| lsp_error_map(ErrorCode::InvalidPosition))?;

        let signatures = signature::get_signature_help(&location, &self.parser);

        Ok(signatures)
    }

    pub fn syntax_tree(&self) -> Result<String, ResponseError> {
        Ok(format!("{:#?}", self.parser.tree()))
    }

    pub fn format(
        &self,
        _context: Arc<Context>,
        workspace_root: &str,
    ) -> Result<Option<Vec<TextEdit>>, ResponseError> {
        let options = get_format_options(workspace_root);
        let formatted = fmt::run(self.parser.tree(), &options);
        let position = self.parser.position();
        let diff = super::diff::diff(&format!("{}", self.parser.tree()), &format!("{formatted}"));

        let ret = diff
            .into_iter()
            .map(|diff| {
                let start = position
                    .get_position_for_size(&(diff.start as u32).into())
                    .ok_or_else(|| lsp_error_map(ErrorCode::InvalidPosition))?
                    .into();

                let end = if let Some(end) = diff.end {
                    position
                        .get_position_for_size(&(end as u32).into())
                        .ok_or_else(|| lsp_error_map(ErrorCode::InvalidPosition))?
                        .into()
                } else {
                    start
                };

                let range = Range::new(start, end);
                Ok(TextEdit {
                    range,
                    new_text: diff.text,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(ret))
    }

    pub fn analysis(
        &self,
        context: Arc<Context>,
        range: Option<DocumentRange>,
    ) -> Result<String, ResponseError> {
        let range = range
            .and_then(|r| self.parser.position().range_to_text_range(&r))
            .unwrap_or_else(|| self.parser.text_range());

        let tokens = self.parser.tokens_in_range(range).filter(|t| {
            !(matches!(t.kind(), SyntaxKind::METADATA)
                || ast::find_parent(t, SyntaxKind::METADATA).is_some())
        });
        let asm = self.parser.reconstruct_from_tokens(tokens, &range);
        run_mca(
            asm.as_str(),
            self.parser.architecture(),
            &context.config().analysis,
        )
        .map_err(|e| lsp_error_map(ErrorCode::MCAFailed(e.to_string())))
    }

    pub fn rename(
        &self,
        context: Arc<Context>,
        rename: crate::handler::types::RenameMessage,
    ) -> Result<Option<WorkspaceEdit>, ResponseError> {
        let token = self
            .parser
            .token_at_point(&rename.location.position)
            .ok_or_else(|| lsp_error_map(ErrorCode::TokenNotFound))?;

        if matches!(token.kind(), SyntaxKind::REGISTER) {
            return Err(lsp_error_map(ErrorCode::InvalidToken(String::from(
                "Rename on a register is unsuported",
            ))));
        }

        let make_rename = |parser: &Parser, token: &SyntaxToken| {
            if matches!(token.kind(), SyntaxKind::REGISTER) {
                return None;
            }

            let new_text = if matches!(token.kind(), SyntaxKind::LABEL) {
                format!("{}:", rename.new_name)
            } else {
                rename.new_name.clone()
            };

            let range = parser.position().range_for_token(token);
            Some((parser.uri().clone(), range, new_text))
        };

        let range = references::get_search_range(&self.parser, &token, None);
        let references = if matches!(token.kind(), SyntaxKind::REGISTER_ALIAS) {
            references::find_references_alias_exact(self.parser(), &token, range)
        } else {
            references::find_references(&self.parser, &token, range, true)
        }
        .filter_map(|t| make_rename(&self.parser, &t));

        let included_files = if token_is_local_label(&token) {
            None
        } else {
            let handle_related = |parser: &Parser| {
                let range = parser.text_range();
                if matches!(token.kind(), SyntaxKind::REGISTER_ALIAS) {
                    references::find_references_alias_exact(parser, &token, range)
                } else {
                    references::find_references(parser, &token, range, true)
                }
                .filter_map(|t| make_rename(parser, &t))
                .collect_vec()
            };

            let related = context.related_parsers(false, self.uri.clone(), |parser| {
                handle_related(parser).into_iter()
            });

            Some(related)
        };

        let changes = references
            .chain(included_files.into_iter().flatten())
            .filter_map(|(url, range, new_text)| {
                Some((
                    url,
                    TextEdit {
                        range: range?.into(),
                        new_text,
                    },
                ))
            })
            .into_group_map();

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
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

fn handle_includes(
    includes: Vec<ParsedInclude>,
    context: Arc<Context>,
    uri: &Url,
) -> Vec<(Url, RwLock<AssemblyLanguageServerProtocol>)> {
    includes
        .into_iter()
        .flat_map(|include| {
            let (parser, sub_includes) = split_parsed_include(include);

            let include_uri = parser.uri().clone();
            let inner = handle_includes(sub_includes, context.clone(), &include_uri);
            context.add_include(uri.clone().to_string(), include_uri.to_string());

            inner.into_iter().chain(std::iter::once((
                include_uri.clone(),
                RwLock::new(AssemblyLanguageServerProtocol {
                    parser,
                    uri: include_uri,
                    version: 0,
                }),
            )))
        })
        .collect_vec()
}

fn get_format_options(workspace_root: &str) -> FormatOptions {
    let mut path = PathBuf::from(workspace_root);
    path.push(".asmfmt.toml");
    if let Ok(data) = std::fs::read_to_string(path) {
        toml::from_str(&data).unwrap_or_default()
    } else {
        Default::default()
    }
}

impl<'s> LabelNode<'s> {
    fn to_document_symbol(&self, position: &PositionInfo) -> Option<DocumentSymbol> {
        let token = find_kind_index(self.syntax(), 1, SyntaxKind::LABEL)?.into_token()?;
        let node = self.syntax();

        Some(DocumentSymbol {
            name: token.text().to_string(),
            detail: None,
            kind: SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            range: position.range_for_node(node).unwrap().into(),
            selection_range: position.range_for_node(node).unwrap().into(),
            children: self
                .sub_labels()
                .map(|s| LocalLabelNode::cast(&s).and_then(|s| s.to_document_symbol(position)))
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
            kind: SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            range: position.range_for_node(node).unwrap().into(),
            selection_range: position.range_for_node(node).unwrap().into(),
            children: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use base::Architecture;
    use lsp_types::{
        DocumentHighlight, DocumentSymbol, DocumentSymbolResponse, GotoDefinitionResponse, Position,
    };
    use pretty_assertions::assert_eq;

    use crate::config::LSPConfig;
    use crate::types::DocumentRange;

    use super::*;

    fn setup_actor(ctx: Arc<Context>, actor: AssemblyLanguageServerProtocol) {
        let uri = actor.uri.clone();
        ctx.actors.write().insert(uri, RwLock::new(actor));
    }

    macro_rules! get_response {
        ($ctx:ident, $method:ident, $($args:expr),*) => {
            $ctx.actors
                .read()
                .get(&Url::parse("file://temp").unwrap())
                .unwrap()
                .read()
                .$method($ctx.clone(), $($args,)*)
                .unwrap()
        };
    }

    #[test]
    fn test_goto_definition_with_label() {
        let ctx: Arc<Context> = Default::default();

        let actor = AssemblyLanguageServerProtocol::new(
            ctx.clone(),
            r#"entry:
    b entry
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            0,
        );
        setup_actor(ctx.clone(), actor);

        let expected = GotoDefinitionResponse::Array(vec![Location {
            uri: Url::parse("file://temp").unwrap(),
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 6),
            },
        }]);

        let response = get_response!(
            ctx,
            goto_definition,
            DocumentPosition { line: 1, column: 8 }
        );

        assert_eq!(expected, response);
    }

    #[test]
    fn test_goto_definition_with_label_not_first() {
        let ctx: Arc<Context> = Default::default();

        let actor = AssemblyLanguageServerProtocol::new(
            ctx.clone(),
            r#"b entry
entry:
    b entry
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            0,
        );
        setup_actor(ctx.clone(), actor);

        let expected = GotoDefinitionResponse::Array(vec![Location {
            uri: Url::parse("file://temp").unwrap(),
            range: Range {
                start: Position::new(1, 0),
                end: Position::new(1, 6),
            },
        }]);

        let response = get_response!(
            ctx,
            goto_definition,
            DocumentPosition { line: 2, column: 8 }
        );

        assert_eq!(expected, response);
    }

    #[test]
    fn test_goto_definition_with_label_not_defined() {
        let ctx: Arc<Context> = Default::default();

        let actor = AssemblyLanguageServerProtocol::new(
            ctx.clone(),
            r#"entry:
    b somewhere
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            0,
        );
        setup_actor(ctx.clone(), actor);

        let expected = GotoDefinitionResponse::Array(vec![]);

        let response = get_response!(
            ctx,
            goto_definition,
            DocumentPosition { line: 1, column: 8 }
        );

        assert_eq!(expected, response);
    }

    #[test]
    fn test_goto_definition_on_opcode() {
        let ctx: Arc<Context> = Default::default();

        let actor = AssemblyLanguageServerProtocol::new(
            ctx.clone(),
            r#"entry:
    stp x20, x21, [sp, -32]!
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            0,
        );
        setup_actor(ctx.clone(), actor);

        let expected = GotoDefinitionResponse::Array(vec![]);

        let response = get_response!(
            ctx,
            goto_definition,
            DocumentPosition { line: 1, column: 6 }
        );

        assert_eq!(expected, response);
    }

    #[test]
    fn test_goto_definition_with_not_on_token() {
        let ctx: Arc<Context> = Default::default();

        let actor = AssemblyLanguageServerProtocol::new(
            ctx.clone(),
            r#"entry:
    stp x20, x21, [sp, -32]!
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            0,
        );
        setup_actor(ctx.clone(), actor);

        let expected = GotoDefinitionResponse::Array(vec![]);

        let response = get_response!(
            ctx,
            goto_definition,
            DocumentPosition { line: 1, column: 7 }
        );

        assert_eq!(expected, response);
    }

    #[test]
    fn test_find_references() {
        let ctx: Arc<Context> = Default::default();

        let actor = AssemblyLanguageServerProtocol::new(
            ctx.clone(),
            r#"entry:
    b entry
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            0,
        );
        setup_actor(ctx.clone(), actor);

        let expected = vec![Location {
            uri: Url::parse("file://temp").unwrap(),
            range: Range {
                start: Position::new(1, 6),
                end: Position::new(1, 11),
            },
        }];

        let response = get_response!(
            ctx,
            find_references,
            DocumentPosition { line: 1, column: 8 },
            false
        );

        assert_eq!(expected, response);
    }

    #[test]
    fn test_find_references_numeric() {
        let ctx: Arc<Context> = Default::default();

        let actor = AssemblyLanguageServerProtocol::new(
            ctx.clone(),
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
        );
        setup_actor(ctx.clone(), actor);

        let response = get_response!(
            ctx,
            find_references,
            DocumentPosition {
                line: 2,
                column: 25,
            },
            false
        );

        let expected: Vec<Location> = vec![];

        assert_eq!(response, expected);
    }

    #[test]
    fn test_find_references_instruction() {
        let ctx: Arc<Context> = Default::default();

        let actor = AssemblyLanguageServerProtocol::new(
            ctx.clone(),
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
        );
        setup_actor(ctx.clone(), actor);

        let response = get_response!(
            ctx,
            find_references,
            DocumentPosition { line: 3, column: 5 },
            false
        );

        let expected: Vec<Location> = vec![];

        assert_eq!(expected, response);
    }

    #[test]
    fn test_document_symbols() {
        let ctx: Arc<Context> = Default::default();

        let actor = AssemblyLanguageServerProtocol::new(
            ctx.clone(),
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
        );
        setup_actor(ctx.clone(), actor);

        let expected = DocumentSymbolResponse::Nested(
            [
                DocumentSymbol {
                    name: "entry:".to_string(),
                    detail: None,
                    kind: SymbolKind::FUNCTION,
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
                        kind: SymbolKind::FUNCTION,
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
                    kind: SymbolKind::FUNCTION,
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

        let response = get_response!(ctx, document_symbols,);

        assert_eq!(expected, response);
    }

    #[test]
    fn test_document_highlight_label() {
        let ctx: Arc<Context> = Default::default();

        let actor = AssemblyLanguageServerProtocol::new(
            ctx.clone(),
            r#"entry:
    b entry
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            0,
        );
        setup_actor(ctx.clone(), actor);

        let expected = vec![
            DocumentHighlight {
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 6),
                },
                kind: Some(DocumentHighlightKind::TEXT),
            },
            DocumentHighlight {
                range: Range {
                    start: Position::new(1, 6),
                    end: Position::new(1, 11),
                },
                kind: Some(DocumentHighlightKind::TEXT),
            },
        ];

        let response = get_response!(
            ctx,
            document_highlight,
            DocumentPosition { line: 1, column: 8 }
        );

        assert_eq!(expected, response);
    }

    #[test]
    fn test_document_semantic() {
        let ctx: Arc<Context> = Default::default();

        let actor = AssemblyLanguageServerProtocol::new(
            ctx.clone(),
            r#"entry:
    stp	x29, x30, [sp, -32]!
    b entry
// lsp-asm-architecture: AArch64"#,
            Url::parse("file://temp").unwrap(),
            0,
        );
        setup_actor(ctx.clone(), actor);

        let expected = SemanticTokensResult::Tokens(SemanticTokens {
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

        let response = get_response!(ctx, get_semantic_tokens, None);

        assert_eq!(expected, response);
    }

    #[test]
    fn test_invalid_versions() {
        let ctx: Arc<Context> = Arc::new(Context::new(
            LSPConfig {
                architecture: Architecture::AArch64,
                ..Default::default()
            },
            String::from(""),
        ));

        let mut lsp = AssemblyLanguageServerProtocol::new(
            ctx.clone(),
            "str x1, [sp, #80]",
            Url::parse("file://test").unwrap(),
            0,
        );

        assert!(lsp
            .update(
                ctx.clone(),
                5,
                vec![DocumentChange {
                    text: String::from("// test"),
                    range: Some(DocumentRange {
                        start: DocumentPosition { line: 0, column: 0 },
                        end: DocumentPosition { line: 0, column: 0 },
                    }),
                }],
            )
            .is_ok());
        assert!(lsp
            .update(
                ctx.clone(),
                5,
                vec![DocumentChange {
                    text: String::from("// te"),
                    range: Some(DocumentRange {
                        start: DocumentPosition { line: 0, column: 0 },
                        end: DocumentPosition { line: 0, column: 0 },
                    }),
                }],
            )
            .is_err());
        assert!(lsp
            .update(
                ctx.clone(),
                3,
                vec![DocumentChange {
                    text: String::from("// te"),
                    range: Some(DocumentRange {
                        start: DocumentPosition { line: 0, column: 0 },
                        end: DocumentPosition { line: 0, column: 0 },
                    }),
                }],
            )
            .is_err());

        assert!(lsp
            .update(
                ctx,
                6,
                vec![DocumentChange {
                    text: String::from("// test more"),
                    range: Some(DocumentRange {
                        start: DocumentPosition { line: 0, column: 0 },
                        end: DocumentPosition { line: 0, column: 0 },
                    }),
                }],
            )
            .is_ok());
    }
}
