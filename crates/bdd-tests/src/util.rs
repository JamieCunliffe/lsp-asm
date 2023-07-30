#![allow(deprecated)]
use base::Architecture;
use itertools::Itertools;
use lsp_asm::config::LSPConfig;
use lsp_asm::diagnostics::Error;
use lsp_asm::handler::semantic;
use lsp_asm::types::{CompletionItem, CompletionKind, DocumentLocation, DocumentRange};
use lsp_types::{
    CodeAction, CodeActionOrCommand, CodeLens, Command, CompletionList, DocumentHighlight,
    DocumentHighlightKind, DocumentSymbol, Documentation, InlayHint, Location, OneOf,
    ParameterInformation, ParameterLabel, PublishDiagnosticsParams, Range, SemanticToken,
    SignatureHelp, SignatureInformation, SymbolKind, TextDocumentEdit, TextEdit, Url,
    WorkspaceEdit,
};
use std::collections::HashMap;
use std::path::Path;

use crate::file::FileUrl;
use crate::position::PositionString;

pub(crate) fn parse_config(rows: &[Vec<String>]) -> LSPConfig {
    let mut config: LSPConfig = Default::default();
    for row in rows.iter().skip(1) {
        let key = row[0].as_str();
        let value = row[1].as_str();
        match key {
            "architecture" => config.architecture = Architecture::from(value),
            "codelens::loc_enabled" => config.codelens.loc_enabled = value.parse::<bool>().unwrap(),
            key => panic!("Unknown configuration parameter: {key}"),
        }
    }
    config
}

pub(crate) fn make_doc_location_vec(
    table: &[Vec<String>],
    file: &FileUrl,
) -> Vec<DocumentLocation> {
    table
        .iter()
        .skip(1)
        .map(|cols| DocumentLocation {
            uri: cols
                .get(2)
                .map(|f| file_to_uri(f.as_str()))
                .unwrap_or_else(|| file.clone().into()),
            range: DocumentRange {
                start: PositionString::from_string(cols.get(0).unwrap().into()).into(),
                end: PositionString::from_string(cols.get(1).unwrap().into()).into(),
            },
        })
        .collect::<Vec<_>>()
}

pub(crate) fn make_lsp_doc_location(file: &FileUrl, table: &[Vec<String>]) -> Vec<Location> {
    make_doc_location_vec(table, file)
        .drain(..)
        .map(|range| range.into())
        .collect()
}

pub(crate) fn make_doc_symbol(table: &[Vec<String>]) -> Vec<DocumentSymbol> {
    let mut symbols: HashMap<u32, DocumentSymbol> = HashMap::new();

    for row in table.iter().skip(1) {
        let symbol = DocumentSymbol {
            name: row.get(1).unwrap().to_string(),
            detail: None,
            kind: match row.get(2).unwrap().as_str() {
                "function" => SymbolKind::FUNCTION,
                kind => panic!("Unknown kind: {kind}"),
            },
            tags: None,
            deprecated: None,
            range: PositionString::from_string(row.get(3).unwrap().into()).into(),
            selection_range: PositionString::from_string(row.get(4).unwrap().into()).into(),
            children: None,
        };
        let pid = row
            .get(5)
            .unwrap()
            .parse::<u32>()
            .unwrap_or_else(|_| row.get(0).unwrap().parse().unwrap());

        if let Some(parent) = symbols.get_mut(&pid) {
            if let Some(c) = parent.children.as_mut() {
                c.push(symbol);
            } else {
                parent.children = Some(vec![symbol]);
            }
        } else {
            symbols.insert(pid, symbol);
        }
    }

    let mut v = symbols.values().cloned().collect::<Vec<_>>();
    v.sort_by(|a, b| a.range.start.partial_cmp(&b.range.start).unwrap());
    v
}

pub(crate) fn make_doc_highlight(table: &[Vec<String>]) -> Vec<DocumentHighlight> {
    table
        .iter()
        .skip(1)
        .map(|row| DocumentHighlight {
            range: PositionString::from_string(row.get(0).unwrap().into()).into(),
            kind: match row.get(1).unwrap().as_str() {
                "text" => Some(DocumentHighlightKind::TEXT),
                kind => panic!("Unknown kind: {kind}"),
            },
        })
        .collect()
}

pub(crate) fn make_semantic(table: &[Vec<String>]) -> Vec<SemanticToken> {
    table
        .iter()
        .skip(1)
        .map(|row| SemanticToken {
            delta_line: row.get(0).unwrap().parse().unwrap(),
            delta_start: row.get(1).unwrap().parse().unwrap(),
            length: row.get(2).unwrap().parse().unwrap(),
            token_type: match row.get(3).unwrap().as_str() {
                "opcode" => semantic::OPCODE_INDEX,
                "string" => semantic::STRING_INDEX,
                "number" => semantic::NUMERIC_INDEX,
                "directive" => semantic::DIRECTIVE_INDEX,
                "comment" => semantic::COMMENT_INDEX,
                "register" => semantic::REGISTER_INDEX,
                "gp-register" => semantic::GP_REGISTER_INDEX,
                "fp-register" => semantic::FP_REGISTER_INDEX,
                "metadata" => semantic::METADATA_INDEX,
                "label" => semantic::LABEL_INDEX,
                ty => panic!("Unknown token type: {ty}"),
            },
            token_modifiers_bitset: u32::from_str_radix(row.get(4).unwrap(), 2).unwrap(),
        })
        .collect()
}

pub(crate) fn make_codelens(table: &[Vec<String>]) -> Option<Vec<CodeLens>> {
    (table.len() > 1).then(|| {
        table
            .iter()
            .skip(1)
            .map(|row| CodeLens {
                range: PositionString::from_string(row.get(0).unwrap().into()).into(),
                command: Some(Command {
                    title: row.get(1).unwrap().into(),
                    command: row.get(2).unwrap().into(),
                    arguments: None,
                }),
                data: None,
            })
            .collect()
    })
}

pub(crate) fn make_completion(table: &[Vec<String>]) -> CompletionList {
    let items = table
        .iter()
        .skip(1)
        .map(|row| CompletionItem {
            text: row.get(0).unwrap().parse().unwrap(),
            details: row.get(1).unwrap().replace("%PIPE%", "|"),
            documentation: row.get(3).map(String::from),
            kind: match row.get(2).unwrap().as_str() {
                "mnemonic" => CompletionKind::Mnemonic,
                "label" => CompletionKind::Label,
                "register" => CompletionKind::Register,
                "text" => CompletionKind::Text,
                kind => panic!("Unknown completion kind: {kind}"),
            },
        })
        .map(|i| i.into())
        .collect::<Vec<_>>();

    CompletionList {
        is_incomplete: true,
        items,
    }
}

pub(crate) fn make_signature_help(table: &[Vec<String>]) -> SignatureHelp {
    let signatures = table
        .iter()
        .skip(1)
        .map(|signature| SignatureInformation {
            label: signature.get(2).unwrap().replace("%PIPE%", "|"),
            documentation: signature
                .get(3)
                .map(|x| Documentation::String(x.to_string())),
            parameters: Some(
                signature
                    .get(4)
                    .unwrap()
                    .split("%%NEXT%%")
                    .zip(signature.get(5).unwrap().split("%%NEXT%%"))
                    .map(|(label, documentation)| ParameterInformation {
                        label: ParameterLabel::Simple(label.replace("%PIPE%", "|")),
                        documentation: Some(Documentation::String(documentation.to_string())),
                    })
                    .collect(),
            ),
            active_parameter: signature.get(1).map(|x| x.parse().unwrap()),
        })
        .collect::<Vec<_>>();

    let active = table.iter().skip(1).enumerate().find_map(|(idx, sig)| {
        sig.get(0)
            .map(|x| x == "*")
            .unwrap_or(false)
            .then_some(idx as u32)
    });

    let active_parameter = active
        .and_then(|a| signatures.get(a as usize).map(|s| s.active_parameter))
        .flatten();

    SignatureHelp {
        signatures,
        active_signature: active,
        active_parameter,
    }
}

pub(crate) fn get_errors(table: &[Vec<String>], uri: Url) -> PublishDiagnosticsParams {
    PublishDiagnosticsParams {
        uri,
        diagnostics: table
            .iter()
            .skip(1)
            .map(|err| Error {
                file: Default::default(),
                line: err.get(0).unwrap().parse::<u32>().unwrap() - 1, // Subtract one as LSP is 0 based
                column: err.get(1).unwrap().parse::<u32>().unwrap(),
                level: err.get(2).unwrap().as_str().into(),
                code: Default::default(),
                description: err.get(3).unwrap().to_string(),
            })
            .map(Into::into)
            .collect(),
        version: None,
    }
}

pub(crate) fn file_to_uri(file: &str) -> Url {
    || -> Option<Url> {
        let file = Path::new(file).canonicalize().ok()?;
        Url::from_file_path(file).ok()
    }()
    .unwrap_or_else(|| {
        let name = format!(
            "{}{}",
            std::env::current_dir()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap(),
            file
        );
        let path = Path::new(&name);

        if path.exists() {
            Url::from_file_path(path).unwrap()
        } else {
            Url::parse(&format!("file://{file}")).unwrap()
        }
    })
}

pub(crate) fn make_workspace_edit(table: &[Vec<String>]) -> WorkspaceEdit {
    let changes = table
        .iter()
        .skip(1)
        .map(|cols| {
            let edit = TextEdit {
                range: DocumentRange {
                    start: PositionString::from_string(cols.get(0).unwrap().into()).into(),
                    end: PositionString::from_string(cols.get(1).unwrap().into()).into(),
                }
                .into(),
                new_text: cols.get(3).unwrap().to_string(),
            };
            (file_to_uri(cols.get(2).unwrap()), edit)
        })
        .into_group_map();

    WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    }
}

pub(crate) fn make_codeaction(rows: &[Vec<String>]) -> Vec<CodeActionOrCommand> {
    rows.iter()
        .skip(1)
        .map(|cols| {
            CodeActionOrCommand::CodeAction(CodeAction {
                title: cols.get(1).unwrap().to_string(),
                kind: None,
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: None,
                    document_changes: Some(lsp_types::DocumentChanges::Edits(vec![
                        TextDocumentEdit {
                            text_document: lsp_types::OptionalVersionedTextDocumentIdentifier {
                                uri: file_to_uri(cols.get(0).unwrap()),
                                version: None,
                            },
                            edits: vec![OneOf::Left(TextEdit {
                                range: Range::new(
                                    PositionString::from_string(cols.get(2).unwrap().into()).into(),
                                    PositionString::from_string(cols.get(3).unwrap().into()).into(),
                                ),
                                new_text: cols.get(4).unwrap().replace(r#"{\n}"#, "\n"),
                            })],
                        },
                    ])),
                    change_annotations: None,
                }),
                command: None,
                is_preferred: None,
                disabled: None,
                data: None,
            })
        })
        .collect_vec()
}

pub(crate) fn make_inlay_hint(rows: &[Vec<String>]) -> Vec<InlayHint> {
    rows.iter()
        .skip(1)
        .map(|cols| InlayHint {
            position: PositionString::from_string(cols.get(0).unwrap().to_string()).into(),
            label: lsp_types::InlayHintLabel::String(cols.get(1).unwrap().to_string()),
            kind: None,
            text_edits: None,
            tooltip: None,
            padding_left: None,
            padding_right: None,
            data: None,
        })
        .collect()
}
